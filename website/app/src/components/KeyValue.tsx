import {
  Skeleton,
  Stack,
  Typography
} from "@mui/joy";
import * as React from "react";

export function KeyValue(props: {
  keyName: string;
  value: React.JSX.Element | string;
}) {
  return (
    <Stack direction="column">
      <Typography
        textColor="inherit"
        sx={{
          opacity: 0.5,
          mb: -0.5,
        }}
      >
        {props.keyName}
      </Typography>
      {props.value ?? (
        <Typography>
          <Skeleton>12345</Skeleton>
        </Typography>
      )}
    </Stack>
  );
}
