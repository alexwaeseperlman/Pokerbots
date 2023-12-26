import { TeamWithMembers } from "@bindings/TeamWithMembers";
import { User } from "@bindings/User";
import { Avatar, Stack, Typography } from "@mui/joy";
import { apiUrl } from "../../../state";
import React from "react";
import { useNavigate } from "react-router-dom";

export default function TeamCard(props: {
  team: TeamWithMembers<User>;
  variant: "small" | "large";
}) {
  const avatarSize = props.variant == "large" ? 64 : 32;
  const navigate = useNavigate();
  return (
    <Stack direction="row" gap={2} mb={4}>
      <Avatar
        key="avatar"
        sx={{
          width: avatarSize,
          height: avatarSize,
        }}
        src={`${apiUrl}/pfp?id=${props.team.id}`}
      />
      <Stack direction="column">
        <Typography
          color="inherit"
          level={props.variant == "large" ? "h2" : "h3"}
          sx={{
            cursor: "pointer",
            userSelect: "none", // Prevent text selection
          }}
          onClick={() => {
            navigate(`/team/${props.team.id}`);
          }}
        >
          {props.team.name}
        </Typography>
        {props.variant === "large" &&
          props.team.members.map(() => {
            return (
              <Typography color="inherit">
                {props.team.members[0].display_name}
              </Typography>
            );
          })}
      </Stack>
    </Stack>
  );
}
