import React, { useState } from "react";
import Box from "@mui/system/Box";
import CloudUploadOutlinedIcon from "@mui/icons-material/CloudUploadOutlined";
import { apiUrl } from "../state";
import { CircularProgress, Stack, styled } from "@mui/joy";
import { useSnackbar } from "notistack";
import CloudUploadOutlined from "@mui/icons-material/CloudUploadOutlined";

export interface FileUploadProps extends React.PropsWithChildren {
  onUpload: (file: File) => Promise<void>;
}

interface FileUploadOwnerState extends FileUploadProps {
  drag: boolean;
}

const FileUploadRoot = styled("div", {
  name: "FileUpload",
  slot: "root",
})<{ ownerState: FileUploadOwnerState }>(({ theme, ownerState }) => ({
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
  ...(ownerState.drag && {
    backgroundColor: "#c4b7ff",
  }),
}));

export default React.forwardRef<HTMLDivElement, FileUploadProps>(
  function FileUpload(props, ref) {
    const { enqueueSnackbar } = useSnackbar();
    const [drag, setDrag] = useState(false);
    const [uploading, setUploading] = useState(false);
    const handleUpload = (file: File) => {
      setUploading(true);
      props
        .onUpload(file)
        .catch((err) => {
          enqueueSnackbar({
            message: err.toString(),
            variant: "error",
          });
        })
        .finally(() => {
          setUploading(false);
        });
    };

    const id = React.useId();

    return (
      <FileUploadRoot
        ref={ref}
        ownerState={{ drag, ...props }}
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
        }}
      >
        <Box
          sx={{
            color: "black",
            display: "flex",
            alignItems: "center",
            gap: 4,
            flexDirection: "row",
          }}
        >
          <CloudUploadOutlined
            sx={{
              fontSize: 48,
            }}
          />
          {uploading ? (
            <CircularProgress />
          ) : (
            <>
              <Stack direction="column" gap={1}>
                {props.children}
                <label
                  style={{
                    color: "#392889",
                    border: "none",
                    textDecoration: "none",
                    cursor: "pointer",
                  }}
                >
                  Click to select a file
                  <input
                    style={{ display: "none" }}
                    onChange={(e) => {
                      if (e.target.files) handleUpload(e.target.files[0]);
                    }}
                    type="file"
                    id={id}
                    name="bot-file-input"
                  />
                </label>
              </Stack>
            </>
          )}
        </Box>
      </FileUploadRoot>
    );
  }
);
