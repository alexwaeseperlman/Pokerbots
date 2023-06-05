/*use std::{os, process::Command};

use cgroups_rs::{cgroup_builder::CgroupBuilder, cpu::CpuController};

pub struct RestrictedUser {}

pub fn create_user(name: String, root_dir: String) -> Result<(), Box<dyn std::error::Error>> {
    let hier = cgroups_rs::hierarchies::auto();

    let class_id = thread::rng().gen::<u32>();
    // Restrict memory and cpu usage
    let cgroup = CgroupBuilder::new(name.as_str())
        .cpu()
        .shares(33)
        .done()
        .memory()
        .memory_hard_limit(
            std::env::var("MEMORY_LIMIT")
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        )
        .done()
        .network()
        .class_id(class_id)
        .done()
        .build(hier)?;

    // Block all network traffic
    // TODO: Test this
    Command::new("iptables")
        .arg("-A")
        .arg("OUTPUT")
        .arg("-m")
        .arg("cgroup")
        .arg("--cgroup")
        .arg(class_id)
        .arg("-j")
        .arg("DROP")
        .output()?;

    let mut cmd = Command::new("useradd")
        .arg("-m")
        .arg("-d")
        .arg(root_dir)
        .arg(name)
        .output()
        .expect("failed to execute process");
}
*/
