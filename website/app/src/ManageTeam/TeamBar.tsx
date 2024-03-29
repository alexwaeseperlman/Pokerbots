import React, { useState, useLayoutEffect, useEffect } from "react";
import { apiUrl, useTeam, useUser } from "../state";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import AddIcon from "@mui/icons-material/Add";
import Table from "@mui/joy/Table";
import Avatar from "@mui/joy/Avatar";
import CircularProgress from "@mui/joy/CircularProgress";
import FileUpload from "../components/BotUpload";
import { TableButton } from "../components/Tables/GameList/TableButton";
import { Button, Input, TextField, Typography, useTheme } from "@mui/joy";
import EditIcon from "@mui/icons-material/Edit";
import CopyIcon from "@mui/icons-material/ContentCopy";
import { enqueueSnackbar } from "notistack";
import { Team } from "@bindings/Team";
import { User } from "@bindings/User";
import { useReadOnly } from "./state";

function PfpUpload({ team }: { team: Team }) {
  const isReadOnly = useReadOnly();
  const [drag, setDrag] = useState(false);
  const fetchTeam = useTeam(null)[1];

  const [boxWidth, setBoxWidth] = useState(0);
  const [uploading, setUploading] = useState(false);

  const handleUpload = async (f: File) => {
    setUploading(true);
    // TODO: Display errors on these api calls
    await fetch(`${apiUrl}/upload-pfp`, {
      method: "PUT",
      body: f,
      //headers: uploadLink.headers,
    })
      .then(async (res) => {
        const json = await res.json();
        if (json !== null && json.error) {
          enqueueSnackbar(json.error, {
            variant: "error",
          });
        }
      })
      .finally(() => {
        setTimeout(() => {
          fetchTeam();
          setUploading(false);
        }, 100);
      });
  };

  return (
    <Box
      sx={(theme) => ({
        height: "100%",

        display: "flex",
        justifyContent: "center",
        alignItems: "center",
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
          if (isReadOnly) return;
          e.preventDefault();
          setDrag(true);
        }}
        onDragOver={(e) => {
          if (isReadOnly) return;
          e.preventDefault();
          setDrag(true);
        }}
        onDragLeave={(e) => {
          if (isReadOnly) return;
          e.preventDefault();
          setDrag(false);
        }}
        onDrop={(e) => {
          if (isReadOnly) return;
          e.preventDefault();
          handleUpload(e.dataTransfer.files[0]);
          setDrag(false);
        }}
        src={uploading ? "" : `${apiUrl}/pfp?id=${team?.id}`}
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

function CopyableInput({ value }: { value: string }) {
  const ref = React.useRef<HTMLInputElement>(null);
  return (
    <Input
      value={value}
      readOnly
      slotProps={{
        input: {
          ref,
        },
      }}
      size="sm"
      endDecorator={<CopyIcon cursor="pointer" />}
      sx={{
        pointerEvents: "auto",
      }}
      onClick={(e) => {
        const input = ref.current!;
        input.select();
        // modern version of the following command
        navigator.clipboard.writeText(input.value);
        enqueueSnackbar("Copied to clipboard", {
          variant: "success",
        });
      }}
    />
  );
}

export function TeamBar({ teamId }: { teamId: string | null }) {
  const [team, fetchTeam] = useTeam(teamId);
  const [editing, setEditing] = useState(false);
  const theme = useTheme();
  const [user, fetchUser] = useUser();
  const headerRef = React.useRef<HTMLHeadingElement>(null);
  const isReadOnly = useReadOnly();
  if (!team) throw new Error("Cannot render team bar without a team");
  console.log(team);

  useEffect(() => {
    if (team.name) {
      headerRef.current!.textContent = team.name;
    }
  }, [team.name]);
  return (
    <Box
      sx={{
        mt: 4,
        mb: 4,
        flexDirection: "row",
        display: "flex",
        alignItems: "center",
        gap: 4,
        [theme.breakpoints.down("sm")]: {
          flexDirection: "column",
        },
      }}
    >
      <PfpUpload team={team} />
      <Box
        sx={{
          flexDirection: "column",
        }}
      >
        <Box
          sx={(theme) => ({
            display: "flex",
            flexDirection: "row",
            alignItems: "baseline",
          })}
        >
          <Typography
            level="h1"
            ref={headerRef}
            contentEditable={editing}
            suppressContentEditableWarning={true}
            id={`team-name-${team?.id}-${editing}`}
            onFocus={(e) => {
              window.getSelection()?.selectAllChildren(e.target);
            }}
            textColor="inherit"
            onBlur={(e) => {
              setEditing(false);
              fetch(`${apiUrl}/rename-team?to=${e.target.textContent}`).then(
                async (res) => {
                  const json = await res.json();
                  if (json.error) {
                    enqueueSnackbar(json.error, {
                      variant: "error",
                    });
                    return;
                  }
                  fetchTeam();
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
            {team?.name}
          </Typography>
          {isReadOnly || editing || (
            <TableButton
              sx={(theme) => ({
                margin: 1,
              })}
              onClick={() => {
                setEditing(true);
                // set a timeout so that the focus happens after the contenteditable is enabled
                setTimeout(() => {
                  if (headerRef.current) {
                    headerRef.current.focus();
                  }
                }, 5);
              }}
            >
              <EditIcon
                sx={(theme) => ({
                  mr: "4px",
                  [theme.breakpoints.down("sm")]: {
                    mr: "0px",
                  },
                })}
                fontSize="small"
              />
              <Box
                sx={(theme) => ({
                  [theme.breakpoints.down("sm")]: {
                    display: "none",
                  },
                })}
              >
                Rename
              </Box>
            </TableButton>
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
            <Table size="sm" color='primary' variant='solid' sx={{
              background: 'none'
            }}>
              <thead>
                <tr>
                  {/* user/invite */}
                  <td
                    style={{
                      height: 0,
                    }}
                  ></td>
                  {/* leave/kick/delete team */}
                  <td style={{ height: 0 }}></td>
                  {/* make owner */}
                  {team.owner === user?.id && team.members.length > 1 && (
                    <td style={{ height: 0, width: "54px" }}></td>
                  )}
                </tr>
              </thead>
              <tbody>
                {team.members.map((member: User) => (
                  <tr key={member.id}>
                    <td
                      style={{
                        overflow: "hidden",
                      }}
                    >
                      <Typography textColor="inherit" level="title-sm">
                        {member.display_name}
                      </Typography>
                    </td>
                    <Box component={(props: any) => <td {...props}></td>}>
                      {(team.owner === user?.id || member.id === user?.id) &&
                        (isReadOnly || (
                          <TableButton
                            onClick={() => {
                              const confirmed = confirm(
                                `Are you sure you want to ${
                                  member.id == user?.id
                                    ? team.owner == user?.id
                                      ? "delete the team"
                                      : "leave the team"
                                    : `change the owner of this team to ${member.display_name}?`
                                }?`
                              );
                              if (!confirmed) return;

                              if (member.id == user?.id) {
                                if (team.owner == user?.id) {
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
                                  `${apiUrl}/update-owner?user_id=${member.id}`
                                )
                                  .then(async (result) => {
                                    enqueueSnackbar("Made owner", {
                                      variant: "success",
                                    });
                                  })
                                  .catch(() => {
                                    enqueueSnackbar("Failed to make owner", {
                                      variant: "error",
                                    });
                                  })
                                  .finally(() => {
                                    fetchTeam();
                                  });
                              }
                            }}
                          >
                            {member.id === user?.id
                              ? team.owner === user?.id
                                ? "Delete team"
                                : "Leave"
                              : "Make owner"}
                          </TableButton>
                        ))}
                    </Box>
                    {team.owner === user?.id &&
                      member.id !== user?.id &&
                      (isReadOnly || (
                        <Box
                          sx={{
                            backgroundColor: "white",
                          }}
                          component={(props: any) => <td {...props}></td>}
                        >
                          <TableButton
                            onClick={() => {
                              if (
                                !confirm(
                                  `Are you sure you want to kick ${member.display_name}?`
                                )
                              )
                                return;
                              fetch(
                                `${apiUrl}/kick-member?user_id=${member.id}`
                              ).then((response) => {
                                fetchTeam();
                              });
                            }}
                          >
                            Kick
                          </TableButton>
                        </Box>
                      ))}
                  </tr>
                ))}
                {!team.invites
                  ? []
                  : team.invites.map((invite) => (
                      <tr key={invite.code}>
                        <td>
                          <CopyableInput
                            value={`${window.location.origin}/join-team?code=${invite.code}`}
                          />
                        </td>
                        {!isReadOnly && (
                          <td>
                            <TableButton
                              onClick={() => {
                                fetch(
                                  `${apiUrl}/cancel-invite?code=${invite.code}`
                                ).then(() => fetchTeam());
                              }}
                            >
                              Cancel invitation
                            </TableButton>
                          </td>
                        )}
                      </tr>
                    ))}
                {isReadOnly ||
                (team.invites?.length ?? 0) + team.members.length >= 5 ? (
                  []
                ) : (
                  <tr key="create-invite">
                    <Box
                      component={(props: any) => <td {...props}></td>}
                      sx={{
                        alignItems: "center",
                        justifyContent: "left",
                        display: "flex",
                      }}
                    >
                      <TableButton
                        startDecorator={<AddIcon />}
                        onClick={() =>
                          fetch(`${apiUrl}/create-invite`).then(async (a) => {
                            fetchTeam();
                          })
                        }
                      >
                        Add a member
                      </TableButton>
                    </Box>
                  </tr>
                )}
              </tbody>
            </Table>
          </Box>
        </Box>
      </Box>
    </Box>
  );
}
