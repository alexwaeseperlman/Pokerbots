import React, { useState, useLayoutEffect } from "react";
import { apiUrl, pfpEndpoint, useTeam, useUser } from "../state";
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
import { Button, Icon } from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";

function PfpUpload(props: { readonly: boolean }) {
  const [drag, setDrag] = useState(false);
  const [team, fetchTeam] = useTeam();

  const [boxWidth, setBoxWidth] = useState(0);
  const [uploading, setUploading] = useState(false);

  const handleUpload = async (f: File) => {
    setUploading(true);
    // TODO: Display errors on these api calls
    /*const uploadLink = await (
      await fetch(`${apiUrl}/pfp-upload-url?content_length=${f.size}`)
    ).json();*/
    await fetch(`${apiUrl}/upload-pfp` /*uploadLink.url*/, {
      method: "PUT",
      body: f,
      //headers: uploadLink.headers,
    }).finally(() => {
      setTimeout(() => {
        fetchTeam();
        setUploading(false);
      }, 1000);
    });
  };

  return (
    <Box
      sx={(theme) => ({
        height: "100%",
        minWidth: "10px",

        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        position: "relative",
      })}
    >
      <Avatar
        sx={(theme) => ({
          [theme.breakpoints.down("md")]: {
            width: "100px",
            height: "100px",
          },
          height: `150px`,
          width: `150px`,
          flexDirection: "column",
        })}
        onDragEnter={(e) => {
          if (props.readonly) return;
          e.preventDefault();
          setDrag(true);
        }}
        onDragOver={(e) => {
          if (props.readonly) return;
          e.preventDefault();
          setDrag(true);
        }}
        onDragLeave={(e) => {
          if (props.readonly) return;
          e.preventDefault();
          setDrag(false);
        }}
        onDrop={(e) => {
          if (props.readonly) return;
          e.preventDefault();
          handleUpload(e.dataTransfer.files[0]);
          setDrag(false);
        }}
        src={
          uploading
            ? ""
            : `${pfpEndpoint}${team?.id}.png?${
                props.readonly
                  ? ""
                  : Math.floor(
                      Date.now() / 1000
                    ) /* Reset the cache every second */
              }`
        }
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

export function TeamBar(props: { readonly: boolean }) {
  const [team, fetchTeam] = useTeam();
  const [editing, setEditing] = useState(false);
  const user = useUser()[0];
  const headerRef = React.useRef<HTMLHeadingElement>(null);
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
        <PfpUpload readonly={props.readonly} />
        <Box
          sx={{
            flexDirection: "column",
          }}
        >
          <Box display="flex" flexDirection="row" alignItems={"baseline"}>
            <h1
              ref={headerRef}
              contentEditable={editing}
              suppressContentEditableWarning={true}
              id={`team-name-${team?.id}-${editing}`}
              style={{
                margin: "10px",
              }}
              onFocus={(e) => {
                window.getSelection()?.selectAllChildren(e.target);
              }}
              onBlur={(e) => {
                setEditing(false);
                fetch(`${apiUrl}/rename-team?to=${e.target.textContent}`).then(
                  () => {
                    // TODO: handle errors in this fetch
                    setTimeout(() => {
                      fetchTeam().then((team) => {
                        if (team) {
                          headerRef.current!.textContent = team.team_name;
                        }
                      });
                    });
                  }
                );
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  const el = e.target as HTMLElement;
                  el.blur();
                }
              }}
            >
              {team?.team_name}
            </h1>
            {props.readonly || editing || (
              <Box>
                <TableButton
                  sx={{
                    margin: 2,
                  }}
                  onClick={() => {
                    if (!props.readonly) setEditing(true);
                    // set a timeout so that the focus happens after the contenteditable is enabled
                    setTimeout(() => {
                      if (headerRef.current) {
                        headerRef.current.focus();
                      }
                    }, 5);
                  }}
                >
                  <EditIcon sx={{ mr: "4px" }} fontSize="small" />
                  Edit
                </TableButton>
              </Box>
            )}
          </Box>
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
                            {props.readonly || (
                              <TableButton
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
                                          fetchTeam();
                                        }
                                      );
                                    } else {
                                      fetch(`${apiUrl}/leave-team`).then(
                                        (response) => {
                                          fetchTeam();
                                        }
                                      );
                                    }
                                  } else {
                                    fetch(
                                      `${apiUrl}/kick-member?email=${member.email}`
                                    ).then((response) => {
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
                            )}
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
                            value={`${apiUrl}/join-team?invite_code=${invite}`}
                            onClick={(e) => {
                              const input = e.target as HTMLInputElement;
                              input.select();
                              // modern version of the following command
                              navigator.clipboard.writeText(input.value);
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
                              onClick={() => {
                                fetch(
                                  `${apiUrl}/cancel-invite?invite_code=${invite}`
                                ).then(() => fetchTeam());
                              }}
                            >
                              Cancel invitation
                            </TableButton>
                          </TableCell>
                        )}
                      </TableRow>
                    ))}
                    {props.readonly || (
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
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            </Box>
          </Box>
          <Box></Box>
        </Box>
      </Container>
    </Box>
  );
}
