import React from "react";
import Box, { BoxProps } from "@mui/joy/Box";
import {
  Button,
  Card,
  Grid,
  Input,
  Sheet,
  Table,
  Typography,
  styled,
} from "@mui/joy";
import { GameTable } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";
import { useUser } from "../state";
import { Mail } from "@mui/icons-material";
import { BotUpload } from "../ManageTeam/BotUpload";

const Cell = styled("td")(({ theme }) => ({
  padding: theme.spacing(1),
}));

export default function Profile() {
  const user = useUser()[0];
  return (
    <Box>
      <Card>
        <Typography level="h3" mb={2}>
          My profile
        </Typography>
        <Table>
          <thead></thead>
          <tbody>
            <tr>
              <td>
                <Typography>Email</Typography>
              </td>
              <td>
                <Typography startDecorator={<Mail />}>{user?.email}</Typography>
              </td>
            </tr>
            <tr>
              <td colSpan={2}>
                <Box
                  sx={{
                    display: "flex",
                    justifyContent: "flex-end",
                    gap: 1,
                  }}
                >
                  <Button>Change password</Button>
                </Box>
              </td>
            </tr>
            <tr>
              <td>
                <Typography>Name</Typography>
              </td>
              <td>
                <Input size="sm" value={user?.display_name} />
              </td>
            </tr>
            <tr>
              <td>
                <Typography>Country</Typography>
              </td>
              <td>
                <Input size="sm" />
              </td>
            </tr>
            <tr>
              <td>
                <Typography>School</Typography>
              </td>
              <td>
                <Input size="sm" />
              </td>
            </tr>
            <tr>
              <td>
                <Typography>Linkedin</Typography>
              </td>
              <td>
                <Input size="sm" />
              </td>
            </tr>
            <tr>
              <td>
                <Typography>Github</Typography>
              </td>
              <td>
                <Input size="sm" />
              </td>
            </tr>
            <tr>
              <td colSpan={2}>
                <Typography>
                  We'd like to connect you with potential employers.
                </Typography>
                <BotUpload />
              </td>
            </tr>
          </tbody>
          <tfoot>
            <tr>
              <td colSpan={2}>
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
              </td>
            </tr>
          </tfoot>
        </Table>
      </Card>
    </Box>
  );
}
