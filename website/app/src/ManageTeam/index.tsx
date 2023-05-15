import React from "react";
import { useTeam, useUser } from "../state";
import CreateTeam from "./CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";
import MuiTableCell from "@mui/material/TableCell";
import { styled } from "@mui/material/styles";
import Button from "@mui/material/Button";

import { secondary_background } from "../styles.module.css";
import { TeamBar } from "./TeamBar";

const DataGrid = React.lazy(() =>
  import("@mui/x-data-grid").then((mod) => ({ default: mod.DataGrid }))
);

export const TableCell = styled(MuiTableCell)({
  borderBottom: "none",
});
export const TableButton = styled((props) => (
  <Button {...props} disableRipple />
))({
  fontSize: "12px",
  fontWeight: 300,
  textAlign: "left",
  justifyContent: "left",
  textTransform: "none",
  padding: 0,
  cursor: "pointer",
});

export default function ManageTeam() {
  const user = useUser()[0];
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
            flexGrow: 1,
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
