import Container from "@mui/joy/Container";
import React, { useEffect } from "react";
import {
  Button,
  SvgIcon,
  SvgIconProps,
  Box,
  Input,
  FormControl,
  FormLabel,
  Typography,
  Stack,
} from "@mui/joy";
import styled from "@mui/system/styled";
import { ButtonProps, Sheet } from "@mui/joy";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { enqueueSnackbar } from "notistack";
import {
  authUrl,
  useGoogleSigninUrl,
  useMicrosoftSigninUrl,
  useUser,
} from "../state";
import HeaderFooter from "../components/HeaderFooter";
function MicrosoftLogo(props: SvgIconProps) {
  return (
    <SvgIcon {...props}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 23 23">
        <path fill="#f3f3f3" d="M0 0h23v23H0z" />
        <path fill="#f35325" d="M1 1h10v10H1z" />
        <path fill="#81bc06" d="M12 1h10v10H12z" />
        <path fill="#05a6f0" d="M1 12h10v10H1z" />
        <path fill="#ffba08" d="M12 12h10v10H12z" />
      </svg>
    </SvgIcon>
  );
}

function GoogleLogo(props: SvgIconProps) {
  return (
    <SvgIcon {...props}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 186.69 190.5">
        <g transform="translate(1184.583 765.171)">
          <path
            clip-path="none"
            mask="none"
            d="M-1089.333-687.239v36.888h51.262c-2.251 11.863-9.006 21.908-19.137 28.662l30.913 23.986c18.011-16.625 28.402-41.044 28.402-70.052 0-6.754-.606-13.249-1.732-19.483z"
            fill="#4285f4"
          />
          <path
            clip-path="none"
            mask="none"
            d="M-1142.714-651.791l-6.972 5.337-24.679 19.223h0c15.673 31.086 47.796 52.561 85.03 52.561 25.717 0 47.278-8.486 63.038-23.033l-30.913-23.986c-8.486 5.715-19.31 9.179-32.125 9.179-24.765 0-45.806-16.712-53.34-39.226z"
            fill="#34a853"
          />
          <path
            clip-path="none"
            mask="none"
            d="M-1174.365-712.61c-6.494 12.815-10.217 27.276-10.217 42.689s3.723 29.874 10.217 42.689c0 .086 31.693-24.592 31.693-24.592-1.905-5.715-3.031-11.776-3.031-18.098s1.126-12.383 3.031-18.098z"
            fill="#fbbc05"
          />
          <path
            d="M-1089.333-727.244c14.028 0 26.497 4.849 36.455 14.201l27.276-27.276c-16.539-15.413-38.013-24.852-63.731-24.852-37.234 0-69.359 21.388-85.032 52.561l31.692 24.592c7.533-22.514 28.575-39.226 53.34-39.226z"
            fill="#ea4335"
            clip-path="none"
            mask="none"
          />
        </g>
      </svg>
    </SvgIcon>
  );
}

const LoginButton = styled((props: ButtonProps) => (
  <Button variant="soft" color="primary" {...props} />
))(({ theme }) => ({
  flexGrow: 1,
}));

export default function Login() {
  const [params, setParams] = useSearchParams();
  const redirect = params.get("redirect") ?? "/manage-team";
  const navigate = useNavigate();
  const [email, setEmail] = React.useState("");
  const [password, setPassword] = React.useState("");
  const [user, fetchUser] = useUser();
  useEffect(() => {
    if (user) {
      navigate(redirect);
    }
  }, [user]);

  const googleSigninUrl = useGoogleSigninUrl();
  const microsoftSigninUrl = useMicrosoftSigninUrl();

  return (
    <HeaderFooter>
      <Container
        maxWidth="sm"
        sx={{
          gridArea: 'content',
          display: "flex",
          flexDirection: "column",
          alignItems: "stretch",
          justifyContent: "center",
          flexGrow: 1,
          gap: 2,
        }}
      >
        <Typography textColor="inherit" level="h1">
          Log in to your account
        </Typography>
        <Input
          placeholder="Email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
        <Input
          placeholder="Password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              handleSubmit();
            }
          }}
        />
        <Button
          variant="solid"
          onClick={() => {
            handleSubmit();
          }}
        >
          Log in
        </Button>
        <Stack direction="row" gap={2}>
          <LoginButton
            variant="soft"
            onClick={() => {
              navigate(`/forgot-password?redirect=${redirect}`);
            }}
          >
            Forgot your password?
          </LoginButton>
        </Stack>
        <Stack direction="row" gap={2}>
          <LoginButton
            onClick={() => {
              window.location.href =
                googleSigninUrl + "&state=" + encodeURIComponent(redirect);
            }}
            startDecorator={<GoogleLogo />}
          >
            With Google
          </LoginButton>
          <LoginButton
            onClick={() => {
              window.location.href =
                microsoftSigninUrl + "&state=" + encodeURIComponent(redirect);
            }}
            startDecorator={<MicrosoftLogo />}
          >
            With Microsoft
          </LoginButton>
        </Stack>
      </Container>
    </HeaderFooter>
  );
  function handleSubmit() {
    fetch(`${authUrl}/email/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        email,
        password,
      }),
    }).then(async (res) => {
      if (res.status == 200) {
        enqueueSnackbar("Logged in!", {
          variant: "success",
        });

        fetchUser();
      } else {
        enqueueSnackbar(`Failed to log in: ${(await res.json()).error}`, {
          variant: "error",
        });
      }
    });
  }
}
