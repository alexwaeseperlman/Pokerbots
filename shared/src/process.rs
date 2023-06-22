use std::{
    ffi::OsStr,
    os::unix::process::ExitStatusExt,
    process::{ExitStatus, Stdio},
    sync::Arc,
};

use tokio::{
    io,
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::{oneshot::channel, Mutex},
    task::JoinHandle,
    time::sleep,
};

#[derive(Debug)]
pub struct Process {
    pub status: Arc<Mutex<Option<ExitStatus>>>,
    pub output: tokio::io::BufReader<ChildStdout>,
    pub input: tokio::io::BufWriter<ChildStdin>,
    pub err: tokio::io::BufReader<ChildStderr>,
    pub child: Arc<Mutex<Child>>,
    listener: JoinHandle<()>,
    kill_channel: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Process {
    pub async fn new(command: &mut Command) -> Result<Process, io::Error> {
        let (send, recv) = channel::<()>();

        let child = Arc::new(Mutex::new(
            command
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
        ));
        let p = child.clone();
        let mut p = p.lock().await;
        let status: Arc<Mutex<Option<ExitStatus>>> = Arc::new(Mutex::new(None));
        let out = Ok(Process {
            status: status.clone(),
            output: tokio::io::BufReader::new(
                p.stdout
                    .take()
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Unable to get stdout"))?,
            ),
            input: tokio::io::BufWriter::new(
                p.stdin
                    .take()
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Unable to get stdin"))?,
            ),
            err: tokio::io::BufReader::new(
                p.stderr
                    .take()
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Unable to get stderr"))?,
            ),
            child: child.clone(),
            listener: tokio::spawn(async move {
                let child = child.clone();
                let mut child = child.lock().await;
                tokio::select! {
                    out = child.wait() => {
                        let mut status = status.lock().await;
                        *status = Some(out.unwrap_or(ExitStatus::from_raw(1)));
                    }
                    _ = recv => {
                        let mut status = status.lock().await;
                        child.kill().await.unwrap();
                        *status = Some(ExitStatus::from_raw(1));
                    }
                }
            }),
            kill_channel: Some(send),
        });
        // sleep to allow the process to start
        sleep(std::time::Duration::from_millis(1)).await;
        out
    }

    pub async fn sh<T: AsRef<OsStr>>(s: T) -> Result<Process, io::Error> {
        Self::sh_configured(s, |command| command).await
    }

    pub async fn sh_configured<T: AsRef<OsStr>, F: Fn(&mut Command) -> &mut Command>(
        s: T,
        configure: F,
    ) -> Result<Process, io::Error> {
        let mut command = Command::new("sh");
        command.arg("-c").arg(s);
        configure(&mut command);
        Process::new(&mut command).await
    }

    pub fn kill(&mut self) -> bool {
        if let Some(a) = self.kill_channel.take() {
            if let Err(_) = a.send(()) {
                return false;
            }
        }
        return true;
    }

    pub async fn wait(&mut self) -> Result<ExitStatus, io::Error> {
        let Process { listener, .. } = self;
        if listener.is_finished() {
            return Ok(self.status.lock().await.take().ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Listener finished before status was available",
            ))?);
        } else {
            listener.await?;
            return Ok(self.status.lock().await.take().ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Listener finished before status was available",
            ))?);
        }
    }
}

impl Drop for Process {
    // Kill the bot if it is still running
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::{
        io::{AsyncBufReadExt, AsyncReadExt},
        time::sleep,
    };

    #[tokio::test]
    async fn test_process() {
        let mut p = Process::sh("echo hello").await.unwrap();
        let output = &mut p.output;
        let mut s = String::new();
        output.read_line(&mut s).await.unwrap();
        assert_eq!(s, "hello\n");
        assert_eq!(p.wait().await.unwrap(), ExitStatus::from_raw(0));
    }

    #[tokio::test]
    async fn test_sleep() {
        let mut p = Process::sh("sleep 10").await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), p.wait())
            .await
            .expect_err("Process should not have exited");
    }

    #[tokio::test]
    async fn kill_and_read() {
        let mut p = Process::sh("echo test && sleep 3 && echo test")
            .await
            .unwrap();

        // sleep to allow the first line to print
        sleep(std::time::Duration::from_millis(10)).await;
        p.kill();

        let out = &mut p.output;
        let mut buf: Vec<u8> = vec![];
        // TODO: figure out why this waits 3 seconds even though the process is killed
        out.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"test\n");
    }

    #[tokio::test]
    async fn test_process_kill() {
        let mut p = Process::sh("sleep 100").await.unwrap();
        p.kill();
        let mut child = p.child.lock().await;
        tokio::time::timeout(std::time::Duration::from_secs(1), child.wait())
            .await
            .unwrap()
            .unwrap();
    }
}
