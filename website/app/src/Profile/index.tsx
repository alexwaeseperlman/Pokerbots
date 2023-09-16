import React from "react";
import Box, { BoxProps } from "@mui/joy/Box";
import {
  AccordionDetails,
  AccordionSummary,
  Autocomplete,
  Button,
  Card,
  Divider,
  FormControl,
  FormHelperText,
  FormLabel,
  Grid,
  Input,
  Sheet,
  Skeleton,
  Table,
  Typography,
  styled,
} from "@mui/joy";
import { GameTable } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";
import { apiUrl, useProfile, useUser } from "../state";
import { InfoOutlined, Mail } from "@mui/icons-material";
import Accordion from "@mui/joy/Accordion";
import { useNavigate } from "react-router-dom";
import { enqueueSnackbar } from "notistack";
import { UserProfile } from "@bindings/UserProfile";
import { UpdateProfileRequest } from "@bindings/UpdateProfileRequest";
import Resume from "./Resume";

const Cell = styled("td")(({ theme }) => ({
  padding: theme.spacing(1),
}));

export default function Profile() {
  const user = useUser()[0];
  const [profile, fetchProfile] = useProfile();
  const navigate = useNavigate();
  const [email, setEmail] = React.useState<string | null>(null);
  const [schools, setSchools] = React.useState<string[]>([]);

  console.log(email);
  React.useEffect(() => {
    if (!user) {
      navigate("/login?redirect=/profile");
    }
  }, [user]);

  React.useEffect(() => {
    fetch(`${apiUrl}/my-email`).then(async (res) => {
      const json = await res.json();
      if (res.status === 200) {
        setEmail(json);
      } else {
        console.log(json);
      }
    });
  }, [user]);

  React.useEffect(() => {
    fetch(`${apiUrl}/schools`).then(async (res) => {
      const json = await res.json();
      if (res.status === 200) {
        setSchools(json);
      } else {
        console.log(json);
      }
    });
  }, []);

  const [displayName, setDisplayName] = React.useState<string>(
    user?.display_name ?? ""
  );
  const [firstName, setFirstName] = React.useState<string>(
    profile?.first_name ?? ""
  );
  const [lastName, setLastName] = React.useState<string>(
    profile?.last_name ?? ""
  );
  const [country, setCountry] = React.useState<string>(profile?.country ?? "");
  const [school, setSchool] = React.useState<string>(profile?.school ?? "");
  const [github, setGithub] = React.useState<string>(profile?.github ?? "");
  const [linkedin, setLinkedin] = React.useState<string>(
    profile?.linkedin ?? ""
  );

  return (
    <Card>
      <Typography level="h2" mb={2}>
        Your profile
      </Typography>
      <FormControl sx={{ display: { sm: "contents" } }}>
        <FormLabel>Email</FormLabel>
        <Input
          startDecorator={<Mail />}
          value={email ?? "Loading..."}
          readOnly
        />
      </FormControl>

      <FormControl sx={{ display: { sm: "contents" } }}>
        <FormLabel>Display name</FormLabel>
        <Input
          value={displayName}
          onChange={(e) => setDisplayName(e.target.value)}
        />
      </FormControl>

      <FormControl>
        <FormLabel>Name</FormLabel>
        <Box
          sx={{
            display: "flex",
            flexDirection: "row",
            gap: 2,
          }}
        >
          <Input
            sx={{
              flexGrow: 1,
            }}
            placeholder="First name"
            value={firstName}
            onChange={(e) => setFirstName(e.target.value)}
          />
          <Input
            sx={{
              flexGrow: 1,
            }}
            placeholder="Last name"
            value={lastName}
            onChange={(e) => setLastName(e.target.value)}
          />
        </Box>
        <FormHelperText>
          In order to receive prizes and be eligible for the leaderboard, please
          enter your real name.
        </FormHelperText>
      </FormControl>

      <FormControl>
        <FormLabel>Country</FormLabel>
        <Input
          placeholder="Canada"
          value={country}
          onChange={(e) => setCountry(e.target.value)}
        />
      </FormControl>

      <FormControl>
        <FormLabel>School</FormLabel>
        <Autocomplete
          options={schools}
          value={school}
          onChange={(e) => {
            setSchool(schools[e.target.value]);
          }}
          placeholder="McGill University"
        />
        <FormHelperText>
          If you are not a student or your school is not listed, please select
          "Other". Only students at the listed schools are eligible for prizes.
        </FormHelperText>
      </FormControl>

      <Divider role="presentation" />

      <Typography level="h3">Recruiting information (optional)</Typography>
      <Typography level="body-sm">
        We'd like to connect you with our sponsors
      </Typography>

      <FormControl>
        <FormLabel>Linkedin</FormLabel>
        <Input
          value={linkedin}
          onChange={(e) => {
            setLinkedin(e.target.value);
          }}
        />
      </FormControl>

      <FormControl>
        <FormLabel>Github</FormLabel>
        <Input
          value={github}
          onChange={(e) => {
            setGithub(e.target.value);
          }}
        />
      </FormControl>

      <FormControl>
        <FormLabel>Resume</FormLabel>
        <Resume />
      </FormControl>

      <Divider role="presentation" />

      <Box
        sx={{
          display: "flex",
          justifyContent: "flex-end",
          gap: 1,
        }}
      >
        <Button variant="plain">Cancel</Button>
        <Button
          onClick={() => {
            if (
              confirm("Do you confirm that all of your information is correct?")
            ) {
              fetch(`${apiUrl}/profile`, {
                method: "PUT",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  first_name: firstName,
                  last_name: lastName,
                  school,
                  github,
                  linkedin,
                  country,
                  display_name: displayName,
                } as UpdateProfileRequest),
              })
                .then(async (res) => {
                  if (res.status === 200) {
                    enqueueSnackbar("Profile updated!", {
                      variant: "success",
                    });
                    navigate("/manage-team");
                    fetchProfile();
                  } else {
                    const message = await res
                      .json()
                      .then((json) => json.error)
                      .catch(() => "Failed to update profile");
                    enqueueSnackbar(message, {
                      variant: "error",
                    });
                  }
                })
                .catch(console.error);
            }
          }}
        >
          Save
        </Button>
      </Box>
    </Card>
  );
}
