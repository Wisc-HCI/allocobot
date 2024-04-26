import {
  AppBar,
  Card,
  CardHeader,
  CardContent,
  Container,
  CssBaseline,
  IconButton,
  Stack,
  ThemeProvider,
  Toolbar,
  Typography,
  createTheme,
} from "@mui/material";
// import "./App.css";
import { Search } from "@mui/icons-material";
import agentNet from "../../../agent_net.json";
import { mapValues } from "lodash";
import { interpolateRainbow } from "d3-scale-chromatic";

type MetaData = {
  type: string;
  value?: string | string[];
};

type Place = {
  id: string;
  name: string;
  tokens: string;
  metaData: MetaData[];
};

type Signature = {
  type: string;
  value: number | [number, number];
};

type Transition = {
  id: string;
  name: string;
  metaData: MetaData[];
  input: { [key: string]: Signature };
  output: { [key: string]: Signature };
};

type PetriNet = {
  id: string;
  name: string;
  places: { [key: string]: Place };
  transitions: { [key: string]: Transition };
  initialMarking: { [key: string]: number };
  nameLookup: { [key: string]: string };
};

function App() {
  const net: PetriNet = agentNet;
  console.log(net);

  // Generate a color map for entries, based on the name lookup
  const colorKeys = Object.keys(net.nameLookup);
  const sizeColor = colorKeys.length;

  const colorMap: { [key: string]: string } = mapValues(
    net.nameLookup,
    (_, key) => interpolateRainbow(colorKeys.indexOf(key) / sizeColor),
  );

  return (
    <ThemeProvider theme={createTheme({ palette: { mode: "dark" } })}>
      <CssBaseline />
      <AppBar position="fixed">
        <Toolbar>
          <IconButton>
            <Search />
          </IconButton>
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            Allocobot Inspector
          </Typography>
        </Toolbar>
      </AppBar>
      <Container style={{ flexGrow: 1, height: "100vh", overflowY: "scroll" }}>
        <Stack
          direction="column"
          spacing={1}
          sx={{ paddingTop: 10, paddingBottom: 5 }}
        >
          {Object.values(net.places).map((place: Place) => (
            <Card
              href={place.id}
              key={place.id}
              component="a"
              rel="noopener noreferrer"
            >
              <CardHeader title={place.name} />
              <CardContent>
                <Stack direction="column" spacing={1}>
                  {place.metaData.map((metaData: MetaData, i: number) => (
                    <div key={`${place.id}-md${i}`}>
                      <span
                        style={{
                          backgroundColor: "#555",
                          padding: 5,
                          borderRadius: 5,
                          marginRight: 3,
                        }}
                      >
                        {metaData.type}
                      </span>
                      {typeof metaData.value === "string" ? (
                        <span
                          style={{
                            backgroundColor: colorMap[metaData.value],
                            padding: 5,
                            borderRadius: 5,
                          }}
                        >
                          {net.nameLookup[metaData.value]}
                        </span>
                      ) : typeof metaData.value === "object" ? (
                        `: ${metaData.value
                          .map((v) => net.nameLookup[v])
                          .join(", ")}`
                      ) : (
                        ""
                      )}
                    </div>
                  ))}
                </Stack>
              </CardContent>
            </Card>
          ))}
        </Stack>
      </Container>
    </ThemeProvider>
  );
}

export default App;
