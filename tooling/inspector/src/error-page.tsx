import { useRouteError } from "react-router-dom";
import {
  CssBaseline,
  ThemeProvider,
  createTheme,
  Alert,
  Button,
} from "@mui/material";
import { Link } from "react-router-dom";

export default function ErrorPage() {
  const error = useRouteError();
  console.error(error);

  return (
    <ThemeProvider theme={createTheme({ palette: { mode: "dark" } })}>
      <CssBaseline />
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          width: "100vw",
          height: "100vh"
        }}
      >
        <Alert severity="error" variant="filled" action={
            <Link to={"/"}><Button variant="contained">Home</Button></Link>
        }>An unexpected error has occurred.</Alert>
      </div>
    </ThemeProvider>
  );
}
