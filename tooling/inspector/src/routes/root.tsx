import { useState } from "react";
import {
  AppBar,
  Box,
  CssBaseline,
  Divider,
  Drawer,
  IconButton,
  List,
  Toolbar,
  Typography,
  ThemeProvider,
  createTheme,
} from "@mui/material";
import MenuIcon from "@mui/icons-material/Menu";
import { Outlet } from "react-router-dom";
import { transitionsAtom, placesAtom, searchAtom, Search } from "../store";
import { useAtomValue, useAtom } from "jotai";
import NodeListLink from "../NodeListLink";
import { TextInput } from "../TextInput";

const drawerWidth = 240;

export default function Root() {
  // console.log(params);

  const places = useAtomValue(placesAtom);
  const transitions = useAtomValue(transitionsAtom);
  // const [search, setSearch] = useAtom(searchAtom);

  const [search, setSearch] = useState<Search>({ name: null, tags: [] });
  const [mobileOpen, setMobileOpen] = useState(false);

  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  const nameLowercase = search.name?.toLowerCase();

  //   const params = useParams();

  const drawer = (
    <div>
      <Divider>Places</Divider>
      <List>
        {Object.values(places)
          .filter((place) =>
            place.name.toLowerCase().includes(nameLowercase || "")
          )
          .map((place) => (
            <NodeListLink key={place.id} id={place.id} type="place" />
          ))}
      </List>
      <Divider>Transitions</Divider>
      <List>
        {Object.values(transitions)
          .filter((transition) =>
            transition.name.toLowerCase().includes(nameLowercase || "")
          )
          .map((transition) => (
            <NodeListLink
              key={transition.id}
              id={transition.id}
              type="transition"
            />
          ))}
      </List>
    </div>
  );

  return (
    <ThemeProvider theme={createTheme({ palette: { mode: "dark" } })}>
      <CssBaseline />
      <Box sx={{ display: "flex" }}>
        <CssBaseline />
        <AppBar
          position="fixed"
          sx={{
            width: { sm: `calc(100% - ${drawerWidth}px)` },
            ml: { sm: `${drawerWidth}px` },
          }}
        >
          <Toolbar>
            <IconButton
              color="inherit"
              aria-label="open drawer"
              edge="start"
              onClick={handleDrawerToggle}
              sx={{ mr: 2, display: { sm: "none" } }}
            >
              <MenuIcon />
            </IconButton>
            <Typography variant="h6" noWrap component="div">
              Allocobot Inspector
            </Typography>
          </Toolbar>
        </AppBar>
        <Box
          component="nav"
          sx={{ width: { sm: drawerWidth }, flexShrink: { sm: 0 } }}
          aria-label="mailbox folders"
        >
          {/* The implementation can be swapped with js to avoid SEO duplication of links. */}
          <Drawer
            variant="temporary"
            open={mobileOpen}
            onClose={handleDrawerToggle}
            ModalProps={{
              keepMounted: true, // Better open performance on mobile.
            }}
            sx={{
              display: { xs: "block", sm: "none" },
              "& .MuiDrawer-paper": {
                boxSizing: "border-box",
                width: drawerWidth,
              },
            }}
          >
            <Toolbar>
              <TextInput
                value={search.name || undefined}
                onChange={(event) =>
                  setSearch({ ...search, name: event.target.value })
                }
              />
            </Toolbar>
            {drawer}
          </Drawer>
          <Drawer
            variant="permanent"
            sx={{
              display: { xs: "none", sm: "block" },
              "& .MuiDrawer-paper": {
                boxSizing: "border-box",
                width: drawerWidth,
              },
            }}
            open
          >
            <Toolbar>
              <TextInput
                value={search.name || undefined}
                onChange={(event) =>
                  setSearch({ ...search, name: event.target.value })
                }
              />
            </Toolbar>
            {drawer}
          </Drawer>
        </Box>
        <Box
          component="main"
          sx={{
            width: { sm: `calc(100% - ${drawerWidth}px)` },
            height: "100vh",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <Toolbar />
          <Box sx={{ flex: 1, backgroundColor: "#222" }}>
            <Outlet />
          </Box>
        </Box>
      </Box>
    </ThemeProvider>
  );
}
