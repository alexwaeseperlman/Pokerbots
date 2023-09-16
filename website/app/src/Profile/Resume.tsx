import React from "react";
import FileUpload from "../components/BotUpload";
import { enqueueSnackbar } from "notistack";
import { apiUrl } from "../state";
import { Box, Button, ButtonGroup } from "@mui/joy";

export default function Resume() {
  const [resumeStatus, setResumeStatus] = React.useState<boolean>(false);
  const fetchResumeStatus = () => {
    fetch(`${apiUrl}/resume-status`).then(async (res) => {
      if (res.status == 200) setResumeStatus(true);
      else setResumeStatus(false);
    });
  };

  React.useEffect(() => {
    fetchResumeStatus();
  }, []);
  return (
    <>
      <Box
        sx={{
          display: "flex",
          flexDirection: "row",
        }}
      >
        {resumeStatus ? (
          <ButtonGroup>
            <Button
              variant="plain"
              component="a"
              color="primary"
              target="_tab"
              href={`${apiUrl}/resume`}
            >
              Current resume
            </Button>
            <Button
              variant="plain"
              component="a"
              color="danger"
              onClick={() => {
                if (confirm("Are you sure you want to delete your resume?")) {
                  fetch(`${apiUrl}/resume`, {
                    method: "DELETE",
                  }).then(async (res) => {
                    if (res.status === 200) {
                      enqueueSnackbar("Resume deleted", { variant: "success" });
                    } else {
                      enqueueSnackbar("Error deleting resume", {
                        variant: "error",
                      });
                    }
                    setTimeout(() => {
                      fetchResumeStatus();
                    }, 50);
                  });
                }
              }}
            >
              Delete
            </Button>
          </ButtonGroup>
        ) : (
          <p>Resume not uploaded</p>
        )}
      </Box>
      <FileUpload
        onUpload={async (f: File) => {
          const res = await fetch(`${apiUrl}/resume`, {
            method: "PUT",
            body: f,
          });

          if (res.status === 200) {
            enqueueSnackbar("Resume uploaded", { variant: "success" });
            fetchResumeStatus();
          } else {
            const message = await res
              .json()
              .then((json) => json.error)
              .catch(() => "Failed to upload resume");
            enqueueSnackbar(message, { variant: "error" });
          }
        }}
      >
        Drag a pdf of your resume here
      </FileUpload>
    </>
  );
}
