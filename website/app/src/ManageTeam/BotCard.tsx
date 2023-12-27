import { BotWithTeam } from "@bindings/BotWithTeam";
import { BuildStatus } from "@bindings/BuildStatus";
import { Team } from "@bindings/Team";
import { Check } from "@mui/icons-material";
import {
  Box,
  Button,
  ButtonGroup,
  Card,
  CardContent,
  CircularProgress,
  Typography,
} from "@mui/joy";
import * as React from "react";
import { useReadOnly } from "./state";
import { KeyValue } from "../components/KeyValue";

// export type BuildStatus = "Unqueued" | "Queued" | "Building" | "BuildSucceeded" | "PlayingTestGame" | "TestGameSucceeded" | "BuildFailed" | "TestGameFailed";
function BuildStatusChip(props: { status: BuildStatus }) {
  let text = "Unknown status";
  let color = "danger";
  let icon = <></>;

  switch (props.status) {
    case "Unqueued":
      text = "Not queued";
      color = "danger";
      break;
    case "Queued":
      text = "Queued";
      color = "warning";
      icon = <CircularProgress size="sm" />;
      break;
    case "Building":
      text = "Building";
      color = "warning";
      icon = <CircularProgress size="sm" />;
      break;
    case "BuildSucceeded":
      text = "Build succeeded";
      color = "warning";
      icon = <CircularProgress size="sm" />;
      break;
    case "PlayingTestGame":
      text = "Playing test game";
      color = "warning";
      icon = <CircularProgress size="sm" />;
      break;
    case "TestGameSucceeded":
      text = "Test game succeeded";
      color = "success";
      break;
    case "BuildFailed":
      text = "Build failed";
      color = "danger";
      break;
    case "TestGameFailed":
      text = "Test game failed";
      color = "danger";
      break;
  }

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 1,
      }}
    >
      <Typography textColor={color}>{text}</Typography>
      {icon}
    </Box>
  );
}

export default function BotCard(props: {
  bot: BotWithTeam<Team>;
  version: number;
  onSetActive: () => void;
  onDelete: () => void;
}) {
  const isReadOnly = useReadOnly();
  return (
    <Card>
      <CardContent>
        <Typography level="h4">{props.bot.name}</Typography>
      </CardContent>
      <CardContent
        sx={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 1,
        }}
      >
        <KeyValue keyName="Version" value={props.version} />
        <KeyValue
          keyName="Status"
          value={<BuildStatusChip status={props.bot.build_status} />}
        />
        <KeyValue
          keyName="Uploaded by"
          value={props.bot.uploaded_by.display_name}
        />
        <KeyValue
          keyName="Uploaded at"
          value={new Date(
            Number(props.bot.created) * 1000
          ).toLocaleDateString()}
        />
        {(!isReadOnly || props.bot.team.active_bot == props.bot.id) && (
          <Box
            sx={{
              gridColumn: "1 / span 2",
              display: "flex",
              flexDirection: "row",
              gap: 1,
              mt: 2,
            }}
          >
            <Button
              sx={{
                flexGrow: 1,
              }}
              color="primary"
              variant="outlined"
              disabled={isReadOnly || props.bot.team.active_bot == props.bot.id}
              onClick={() => props.onSetActive()}
            >
              {props.bot.team.active_bot == props.bot.id
                ? "Currently active"
                : "Make active"}
            </Button>
            {!isReadOnly && (
              <Button
                sx={{
                  flexGrow: 1,
                }}
                color="danger"
                variant="outlined"
                onClick={() => props.onDelete()}
              >
                Delete
              </Button>
            )}
          </Box>
        )}
      </CardContent>
    </Card>
  );
}
