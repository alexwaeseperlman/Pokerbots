import React from "react";
import { styled } from "@mui/joy/styles";
import Button, { ButtonProps } from "@mui/joy/Button";


export const TableButton = styled((props: ButtonProps) => (
  <Button
    {...props}
    variant="plain"
    sx={{
      color: 'inherit',
      opacity: 0.75,
      whiteSpace: "nowrap",
      background: "none",
      ":hover": {
        background: "#00000040",
      },
      ":active": {
        background: "#00000080",
      },
    }}
    size="sm" />
))(() => ({}));
