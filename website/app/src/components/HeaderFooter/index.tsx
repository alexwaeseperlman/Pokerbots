import { Box, CircularProgress, Sheet } from "@mui/joy";
import * as React from "react";
import { BottomBar, TopBar } from "../AppBar";
import { Suspense } from "react";
import BackgroundImage from "../BackgroundImage";
import bgImage from "./bg.png";

export interface IHeaderFooterProps {
  graphics?: string[];
}

export default function HeaderFooter(
  props: React.PropsWithChildren<IHeaderFooterProps>
) {
  return (
    <Sheet
      sx={{
        flexDirection: "column",
        minHeight: "100vh",
        position: "relative",
        display: "flex",
        background: "linear-gradient(269.89deg,#392889 0%,#191335 100%)",
        pb: 4,
        boxSizing: "border-box",
      }}
      color="primary"
      variant="solid"
    >
      <TopBar />
      <Suspense
        fallback={
          <>
            <Box
              sx={{
                flexGrow: 1,
                display: "flex",
                justifyContent: "center",
                alignItems: "center",
              }}
            >
              <CircularProgress />
            </Box>
          </>
        }
      >
        <Box
          sx={(theme) => ({
            p: 4,
            zIndex: 2,
            display: "grid",
            flexGrow: 1,
            gap: 4,
            [theme.breakpoints.down("md")]: {
              gridTemplateRows: "auto auto 1fr",
              gridTemplateColumns: "1fr",
              gridTemplateAreas: `
                "extra"
                "head"
                "content"
              `,
            },
            [theme.breakpoints.up("md")]: {
              pl: 8,
              gridTemplateRows: "auto 1fr",
              gridTemplateColumns: "3fr 1fr",
              [theme.breakpoints.up("lg")]: {
                gridTemplateColumns: "2fr 1fr",
              },
              gridTemplateAreas: `
                "head extra"
                "content ."
              `,
            },
          })}
        >
          {props.children}
        </Box>
      </Suspense>
      <Box
        sx={{
          position: "absolute",
          bottom: 0,
          width: "100%",
        }}
      >
        <BottomBar />
      </Box>
      <BackgroundImage
        graphics={props.graphics ?? [`url(${bgImage})`]}
        sx={(theme) => ({
          backgroundPosition: "top",
          maxWidth: "100vw",
          maxHeight: "100vh",
          [theme.breakpoints.up("md")]: {
            backgroundPosition: "right",
          },
        })}
      />{" "}
    </Sheet>
  );
}
