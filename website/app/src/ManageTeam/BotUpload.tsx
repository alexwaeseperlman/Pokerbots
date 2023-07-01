import React, { useState } from "react";
import Box from "@mui/system/Box";
import CloudUploadOutlinedIcon from "@mui/icons-material/CloudUploadOutlined";
import { apiUrl } from "../state";
import { CircularProgress, LinearProgress } from "@mui/material";
import { useSnackbar } from "notistack";

export function BotUpload() {
  const { enqueueSnackbar } = useSnackbar();
  const [drag, setDrag] = useState(false);
  const [uploading, setUploading] = useState(false);
  const handleUpload = (file: File) => {
    setUploading(true);
    fetch(`${apiUrl}/upload-bot`, {
      method: "POST",
      body: file,
    })
      .then(async (res) => {
        const json = await res.json();
        console.log(json);
        if (res.status !== 200) {
          enqueueSnackbar({
            message: json.error,
            variant: "error",
          });
        }
      })
      .catch((err) => {
        console.log(err);
        enqueueSnackbar({
          message: err.toString(),
          variant: "error",
        });
      })
      .finally(() => {
        console.log("finally");
        setUploading(false);
      });
  };

  return (
    <Box
      sx={(theme) => ({
        borderRadius: "8px",
        backgroundColor: "white",
        transition: "ease-out 0.2s",
        border: `#999 dashed ${2}px`,
        //width: "188px",
        height: "98px",
        display: "flex",
        padding: "16px",
        gap: "16px",
        //flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
        mb: 2,
        ...(drag && {
          backgroundColor: "#c4b7ff",
        }),
      })}
      onDragEnter={(e) => {
        e.preventDefault();
        setDrag(true);
      }}
      onDragOver={(e) => {
        e.preventDefault();
        setDrag(true);
      }}
      onDragLeave={(e) => {
        e.preventDefault();
        setDrag(false);
      }}
      onDrop={(e) => {
        e.preventDefault();
        handleUpload(e.dataTransfer.files[0]);
        setDrag(false);
        console.log("file dragged");
      }}
    >
      {/*
        <CloudUploadOutlinedIcon />
    */}
      <Box
        style={{
          color: "black",
        }}
      >
        {uploading ? (
          <CircularProgress />
        ) : (
          <>
            Drag and drop a zipped bot here or{" "}
            <label
              style={{
                color: "#392889",
                border: "none",
                textDecoration: "none",
                cursor: "pointer",
              }}
            >
              click to select files
              <input
                style={{ display: "none" }}
                onChange={(e) => {
                  if (e.target.files) handleUpload(e.target.files[0]);
                }}
                type="file"
                id="bot-file-input"
                name="bot-file-input"
              />
            </label>
          </>
        )}
      </Box>
    </Box>
  );
}
