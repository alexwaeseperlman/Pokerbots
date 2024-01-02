import { styled } from "@mui/system";

export default styled("div")<{ graphics: string[] }>(({ theme, graphics }) => ({
  backgroundImage: graphics.join(", "),
  //filter: "grayscale(100%)",
  opacity: 0.4,
  backgroundPosition: "center",
  position: "absolute",
  top: 0,
  left: 0,
  backgroundSize: "contain",
  backgroundRepeat: "no-repeat",
  mixBlendMode: "lighten",
  width: "100%",
  maxWidth: "100vw",
  height: "100%",
  overflow: "hidden",
  pointerEvents: "none",
  zIndex: 1,
  maskImage: "radial-gradient(circle, black, transparent 50%)",
}));
