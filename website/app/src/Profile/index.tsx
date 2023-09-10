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
import { apiUrl, useUser } from "../state";
import { InfoOutlined, Mail } from "@mui/icons-material";
import FileUpload from "../components/BotUpload";
import Accordion from "@mui/joy/Accordion";
import { useNavigate } from "react-router-dom";

const Cell = styled("td")(({ theme }) => ({
  padding: theme.spacing(1),
}));

export default function Profile() {
  const user = useUser()[0];
  const navigate = useNavigate();
  const [email, setEmail] = React.useState<string | null>(null);
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

  return (
    <Card>
      <Typography level="h2" mb={2}>
        Your profile
      </Typography>
      <FormControl sx={{ display: { sm: "contents" } }}>
        <FormLabel>Email</FormLabel>
        {email ? (
          <Input startDecorator={<Mail />} value={email} readOnly />
        ) : (
          <Skeleton component={"span"} width={"100%"} height={"32px"} />
        )}
      </FormControl>

      <FormControl>
        <FormLabel>Name</FormLabel>
        <Input />
        <FormHelperText>
          In order to receive prizes and be eligible for the leaderboard, please
          enter your real name.
        </FormHelperText>
      </FormControl>

      <FormControl>
        <FormLabel>Country</FormLabel>
        <Input />
      </FormControl>

      <FormControl>
        <FormLabel>School</FormLabel>
        <Autocomplete options={["hi"]} />
        <FormHelperText>
          Leave this blank if you are not a student. Note that only students are
          eligible for prizes.
        </FormHelperText>
      </FormControl>

      <Divider role="presentation" />

      <Typography level="h3">Recruiting information</Typography>
      <Typography level="body-sm">
        We'd like to connect you with our sponsors
      </Typography>

      <FormControl>
        <FormLabel>Linkedin</FormLabel>
        <Input />
      </FormControl>

      <FormControl>
        <FormLabel>Github</FormLabel>
        <Input />
      </FormControl>

      <FormControl>
        <FileUpload
          onUpload={(f: File) => {
            return Promise.resolve();
          }}
        >
          Drag your resume here
        </FileUpload>
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
        <Button>Save</Button>
      </Box>
    </Card>
  );
}
