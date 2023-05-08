import React, { useState } from "react";
import { useTeam, useUser } from "../state";
import CreateTeam from "../CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";
import AddIcon from "@mui/icons-material/Add";
import CloudUploadOutlinedIcon from "@mui/icons-material/CloudUploadOutlined";
import {
  Table,
  TableBody,
  TableCell as MuiTableCell,
  TableContainer,
  TableRow,
  styled,
  Button,
} from "@mui/material";

const TableCell = styled(MuiTableCell)({
  borderBottom: "none",
});
const TableButton = styled((props) => <Button {...props} disableRipple />)({
  fontSize: "12px",
  fontWeight: "bold",
  textAlign: "left",
  justifyContent: "left",
  fontWeight: 300,
  textTransform: "none",
  padding: 0,
  cursor: "pointer",
});

function BotUpload() {
  const [drag, setDrag] = useState(false);
  const [inPage, setInPage] = useState(false);

  return (
    <Box
      sx={(theme) => ({
        borderRadius: theme.shape.borderRadius,
        backgroundColor: "white",
        border: `#FFB5C6 solid 2px`,
        width: "188px",
        height: "98px",
        display: "flex",
        padding: "20px",
        gap: "10px",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
        ...(drag && {
          backgroundColor: "#FFB5C6",
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
        <a
          style={{
            color: "#CC385A",
            border: "none",
            textDecoration: "none",
          }}
          href="#"
        >
          click to select files
        </a>
      </Box>
    </Box>
  );
}

function TeamBar() {
  const [team, fetchTeam] = useTeam();
  const user = useUser();
  if (!user || !team)
    throw new Error("Cannot render team bar when not logged in with a team");
  return (
    <Box
      sx={{
        background: "linear-gradient(89.88deg, #CD395C 0%, #E76FBE 100%)",
        color: "white",
        padding: 2,
      }}
    >
      <Container
        sx={{
          flexDirection: "row",
        }}
      >
        <Box
          sx={{
            flexDirection: "column",
          }}
        >
          <h1
            style={{
              margin: "10px",
            }}
          >
            {team?.team_name}
          </h1>
          <Box
            sx={{
              flexDirection: "row",
              display: "flex",
              gap: "10px",
            }}
          >
            <Box display="flex">
              <TableContainer>
                <Table size="small">
                  <TableBody>
                    {team.members.map((member) => (
                      <TableRow>
                        <TableCell
                          sx={{
                            color: "white",
                          }}
                        >
                          {member.display_name}
                        </TableCell>
                        {(team.owner == user.email ||
                          member.email == user.email) && (
                          <TableCell
                            sx={{
                              color: "white",
                            }}
                          >
                            <TableButton
                              sx={{
                                background: "none",
                                border: "none",
                                color: "white",
                              }}
                              onClick={() => {
                                const confirmed = confirm(
                                  `Are you sure you want to ${
                                    member.email == user.email
                                      ? team.owner == user.email
                                        ? "delete the team"
                                        : "leave the team"
                                      : "kick this member"
                                  }?`
                                );
                                if (!confirmed) return;

                                if (member.email == user.email) {
                                  if (team.owner == user.email) {
                                    fetch(`/api/delete-team`).then(
                                      (response) => {
                                        console.log(response);
                                        fetchTeam();
                                      }
                                    );
                                  } else {
                                    fetch(`/api/leave-team`).then(
                                      (response) => {
                                        console.log(response);
                                        fetchTeam();
                                      }
                                    );
                                  }
                                } else {
                                  fetch(
                                    `/api/kick-member?email=${member.email}`
                                  ).then((response) => {
                                    console.log(response);
                                    fetchTeam();
                                  });
                                }
                              }}
                            >
                              {member.email == user.email
                                ? team.owner == user.email
                                  ? "Delete team"
                                  : "Leave"
                                : "Kick"}
                            </TableButton>
                          </TableCell>
                        )}
                      </TableRow>
                    ))}

                    {team.invites.map((invite) => (
                      <TableRow>
                        <TableCell
                          sx={{
                            color: "white",
                          }}
                        >
                          <input
                            value={`${window.location.origin}/api/join-team?invite_code=${invite}`}
                            onClick={(e) => {
                              e.target.select();
                              // modern version of the following command
                              navigator.clipboard.writeText(e.target.value);
                            }}
                            readOnly
                          />
                        </TableCell>
                        {team.owner == user.email && (
                          <TableCell
                            sx={{
                              color: "white",
                            }}
                          >
                            <TableButton
                              sx={{
                                background: "none",
                                border: "none",
                                color: "white",
                              }}
                              onClick={() => {
                                fetch(
                                  `/api/cancel-invite?invite_code=${invite}`
                                ).then(() => fetchTeam());
                              }}
                            >
                              Cancel invite link
                            </TableButton>
                          </TableCell>
                        )}
                      </TableRow>
                    ))}
                    <TableRow>
                      <TableCell
                        sx={{
                          alignItems: "center",
                          justifyContent: "left",
                          display: "flex",
                        }}
                      >
                        <TableButton
                          startIcon={<AddIcon />}
                          sx={{
                            background: "none",
                            border: "none",
                            color: "white",
                          }}
                          onClick={() =>
                            fetch("/api/make-invite").then(() => fetchTeam())
                          }
                        >
                          Add a member
                        </TableButton>
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>
            <BotUpload />
          </Box>
          <Box></Box>
        </Box>
      </Container>
    </Box>
  );
}

export default function ManageTeam() {
  const user = useUser();
  const team = useTeam()[0];

  if (user === undefined) {
    return <div>Loading...</div>;
  }
  if (team && user) {
    return (
      <>
        <TeamBar />
        <br />
        {team.members.map((member) => (
          <>{member.display_name}</>
        ))}
        <form action="/api/delete-team">
          <button
            type="submit"
            onClick={function confirmDelete(event) {
              if (
                confirm(
                  "Are you sure you want to delete your team? This action cannot be undone."
                )
              ) {
                // Proceed with form submission
              } else {
                // Prevent form submission
                event.preventDefault();
              }
            }}
          >
            Delete Team
          </button>
        </form>
      </>
    );
  } else if (user) {
    return <CreateTeam />;
  } else {
    return <Login />;
  }
}
