import React, { useState } from "react";
import { useTeam, useUser } from "../state";
import CreateTeam from "./CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";
import AddIcon from "@mui/icons-material/Add";
import CloudUploadOutlinedIcon from "@mui/icons-material/CloudUploadOutlined";
import TableBody from "@mui/material/TableBody";
import MuiTableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableRow from "@mui/material/TableRow";
import Table from "@mui/material/Table";
import { styled } from "@mui/material/styles";
import Button from "@mui/material/Button";

import { primary_background, secondary_background } from "../styles.module.css";

const DataGrid = React.lazy(() =>
  import("@mui/x-data-grid").then((mod) => ({ default: mod.DataGrid }))
);

const TableCell = styled(MuiTableCell)({
  borderBottom: "none",
});
const TableButton = styled((props) => <Button {...props} disableRipple />)({
  fontSize: "12px",
  fontWeight: 300,
  textAlign: "left",
  justifyContent: "left",
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
        transition: "ease-out 0.2s",
        border: `#c4b7ff solid ${2}px`,
        width: "188px",
        height: "98px",
        display: "flex",
        padding: "20px",
        gap: "10px",
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
            type="file"
            id="bot-file-input"
            name="bot-file-input"
          />
        </label>
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
      className={primary_background}
      sx={{
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
        <Box
          className={secondary_background}
          sx={{
            width: "100%",
            height: "100%",
            padding: "20px",
          }}
        >
          <Container>
            <DataGrid
              columns={[
                { field: "bot-name", headerName: "Bot name", width: 130 },
                { field: "uploaded", headerName: "Uploaded", width: 130 },
                { field: "uploaded-by", headerName: "Uploaded by", width: 130 },
              ]}
              rows={[
                {
                  id: 1,
                  "bot-name": "Bot 1",
                  uploaded: "2021-10-01",
                  "uploaded-by": "User 1",
                },
                {
                  id: 2,
                  "bot-name": "Bot 2",
                  uploaded: "2021-10-02",
                  "uploaded-by": "User 2",
                },
              ]}
            ></DataGrid>
          </Container>
        </Box>
      </>
    );
  } else if (user) {
    return <CreateTeam />;
  } else {
    return <Login />;
  }
}
