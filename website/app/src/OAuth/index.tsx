import React from "react";
import { CircularProgress, Container } from "@mui/joy";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { authUrl, useUser } from "../state";
import { enqueueSnackbar } from "notistack";
import HeaderFooter from "../components/HeaderFooter";
export default function OAuth() {
  const [queryParams, setQueryParams] = useSearchParams();
  const [user, fetchUser] = useUser();
  const params = useParams();
  const navigate = useNavigate();
  React.useEffect(() => {
    fetch(
      `${authUrl}/oauth/${params.provider}/login?code=${queryParams.get(
        "code"
      )}`
    )
      .then(async (res) => {
        if (res.status == 200) {
          enqueueSnackbar("Logged in!", {
            variant: "success",
          });
        } else {
          enqueueSnackbar(`Failed to log in: ${(await res.json()).error}`, {
            variant: "error",
          });
        }
        navigate(queryParams.get("state") ?? "/");
      })
      .finally(() => {
        fetchUser();
      });
  });
  return (
    <HeaderFooter>
      <Container
        sx={{
          gridArea: "content",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <CircularProgress />
      </Container>
    </HeaderFooter>
  );
}
