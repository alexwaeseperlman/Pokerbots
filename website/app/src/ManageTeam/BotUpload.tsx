import React, { useState } from "react";
import Box from "@mui/system/Box";
import CloudUploadOutlinedIcon from "@mui/icons-material/CloudUploadOutlined";
import { apiUrl } from "../state";

export function BotUpload() {
  const [drag, setDrag] = useState(false);
  const [uploading, setUploading] = useState(false);
  const handleUpload = (file: File) => {
    setUploading(true);
    fetch(`${apiUrl}/upload-bot`, {
      method: "POST",
      body: file,
    })
      .then(async (res) => {
        console.log(await res.text());
      })
      .catch((err) => {
        console.log(err);
      })
      .finally(() => {
        setUploading(false);
      });
  };

  return (
    <Box
      sx={(theme) => ({
        borderRadius: "8px",
        backgroundColor: "white",
        transition: "ease-out 0.2s",
        border: `#c4b7ff solid ${2}px`,
        width: "188px",
        height: "98px",
        display: "flex",
        padding: "16px",
        gap: "16px",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
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
      <Box
        style={{
          color: "black",
          display: "flex",
          alignItems: "center",
          gap: "10px",
        }}
      >
        <CloudUploadOutlinedIcon /> Upload bots
      </Box>
      <Box
        style={{
          color: "black",
          fontSize: "12px",
        }}
      >
        Drag and drop files here or{" "}
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
              handleUpload(e.target.files[0]);
            }}
            type="file"
            id="bot-file-input"
            name="bot-file-input"
          />
        </label>
      </Box>
    </Box>
  );
}
