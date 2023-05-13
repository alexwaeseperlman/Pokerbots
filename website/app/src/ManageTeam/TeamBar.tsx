import React, { useState, useLayoutEffect } from "react";
import { apiUrl, usePfpEndpoint, useTeam, useUser } from "../state";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import AddIcon from "@mui/icons-material/Add";
import TableBody from "@mui/material/TableBody";
import TableContainer from "@mui/material/TableContainer";
import TableRow from "@mui/material/TableRow";
import Table from "@mui/material/Table";
import Avatar from "@mui/material/Avatar";
import CircularProgress from "@mui/material/CircularProgress";
import { primary_background } from "../styles.module.css";
import { BotUpload } from "./BotUpload";
import { TableCell, TableButton } from ".";

function PfpUpload() {
  const [drag, setDrag] = useState(false);
  const [team, fetchTeam] = useTeam();
  const pfpEndpoint = usePfpEndpoint();
  console.log(pfpEndpoint);

  const boxRef = React.useRef<HTMLDivElement>(null);
  const [boxWidth, setBoxWidth] = useState(0);
  const [uploading, setUploading] = useState(false);

  const handleUpload = async (f: File) => {
    setUploading(true);
    const uploadLink = await (await fetch(`${apiUrl}/pfp-upload-url`)).json();
    await fetch(uploadLink.url, {
      method: "PUT",
      body: f,
      headers: uploadLink.headers,
    }).finally(() => {
      setTimeout(() => {
        fetchTeam();
        setUploading(false);
      }, 1000);
    });
  };

  // Read the height so we can set the width to make the pfp square
  useLayoutEffect(() => {
    const box = boxRef.current;
    if (!box) return;
    const resizeListener = () => {
      setBoxWidth((box.clientHeight * 5) / 6);
    };
    window.addEventListener("resize", resizeListener);
    resizeListener();
    return () => {
      window.removeEventListener("resize", resizeListener);
    };
  });

  return (
    <Box
      sx={(theme) => ({
        [theme.breakpoints.down("md")]: {
          height: "30%",
        },
        height: "100%",
        minWidth: "10px",

        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        position: "relative",
      })}
      ref={boxRef}
    >
      <Avatar
        sx={(theme) => ({
          height: `${boxWidth}px`,
          width: `${boxWidth}px`,
          flexDirection: "column",
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
        }}
        src={uploading ? "" : `${pfpEndpoint}/${team?.id}.png?${Date.now()}`}
      ></Avatar>

      <Box
        sx={{
          position: "absolute",
          color: "white",
          height: `${boxWidth}px`,
          width: `${boxWidth}px`,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          borderRadius: "100%",
          pointerEvents: "none",
          transition: "0.2s ease-out",
          ...(drag && {
            backgroundColor: "#66666666",
          }),
        }}
      >
        {drag && "Drop an image here"}
        {uploading && <CircularProgress />}
      </Box>
    </Box>
  );
}

export function TeamBar() {
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
          display: "flex",
          alignItems: "center",
          gap: "16px",
          height: "100%",
        }}
      >
        <PfpUpload />
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
                      <TableRow key={member.email}>
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
                                    fetch(`${apiUrl}/delete-team`).then(
                                      (response) => {
                                        console.log(response);
                                        fetchTeam();
                                      }
                                    );
                                  } else {
                                    fetch(`${apiUrl}/leave-team`).then(
                                      (response) => {
                                        console.log(response);
                                        fetchTeam();
                                      }
                                    );
                                  }
                                } else {
                                  fetch(
                                    `${apiUrl}/kick-member?email=${member.email}`
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
                      <TableRow key={invite}>
                        <TableCell
                          sx={{
                            color: "white",
                          }}
                        >
                          <input
                            value={`${apiUrl}/api/join-team?invite_code=${invite}`}
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
                                  `${apiUrl}/api/cancel-invite?invite_code=${invite}`
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
                            fetch(`${apiUrl}/make-invite`).then(() =>
                              fetchTeam()
                            )
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
