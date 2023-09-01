import React from "react";
import { CircularProgress, Container } from "@mui/joy";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { authUrl } from "../state";
import { enqueueSnackbar } from "notistack";
export default function OAuth() {
  const [queryParams, setQueryParams] = useSearchParams();
  const params = useParams();
  const navigate = useNavigate();
  React.useEffect(() => {
    fetch(
      `${authUrl}/oauth/${params.provider}/login?code=${queryParams.get(
        "code"
      )}`
    ).then(async (res) => {
      if (res.status == 200) {
        enqueueSnackbar("Logged in!", {
          variant: "success",
        });
      } else {
        enqueueSnackbar(`Failed to log in: ${(await res.json()).error}`, {
          variant: "error",
        });
      }
      navigate("/");
    });
  });
  return (
    <Container
      sx={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <CircularProgress />
    </Container>
  );
}
