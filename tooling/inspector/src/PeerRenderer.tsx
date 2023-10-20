import {
  Paper,
  Grid,
  styled,
  Tooltip,
  IconButton,
  Chip,
  Stack,
} from "@mui/material";
import { MetaData, colorLookupAtom, nameLookupAtom } from "./store";
import { useAtomValue } from "jotai";
import { Neighbors, Transition, Place } from "./store";
import { ArrowLeft, ArrowRight } from "@mui/icons-material";
import { Link } from "react-router-dom";
import MetaDataPreview from "./MetaDataPreview";
// Renders the metadata for a given node as a table

interface ItemProps {
  color?: string;
}
const Item = styled(Stack)<ItemProps>(({ theme, color }) => ({
  backgroundColor: color
    ? color
    : theme.palette.mode === "dark"
    ? "#555"
    : "#bbb",
  ...theme.typography.body2,
  borderRadius: theme.shape.borderRadius,
  justifyItems: "space-between",
  padding: theme.spacing(1),
  textAlign: "center",
  color: theme.palette.text.secondary,
}));

export default function PeerRenderer({
  neighbors,
}: {
  neighbors: { incoming: Neighbors; outgoing: Neighbors };
}) {
  //   const colorLookup = useAtomValue(colorLookupAtom);
  //   const nameLookup = useAtomValue(nameLookupAtom);

  const incomingPeers = Object.values(neighbors.incoming);
  const outgoingPeers = Object.values(neighbors.outgoing);
  //   console.log(incomingPeers);

  const length = Math.max(incomingPeers.length, outgoingPeers.length);

  return (
    <Grid container spacing={1}>
      <Grid item xs={6}>
        <Item color="#444">Incoming</Item>
      </Grid>
      <Grid item xs={6}>
        <Item color="#444">Outgoing</Item>
      </Grid>
      {Array.from(Array(length).keys()).map((i) => (
        <>
          {incomingPeers[i] ? (
            <Grid item xs={6}>
              <Item
                direction="row"
                alignItems="center"
                justifyContent="space-between"
              >
                <Link
                  to={`/${
                    isTransition(incomingPeers[i][0]) ? "transition" : "place"
                  }s/${incomingPeers[i][0].id}`}
                >
                  <IconButton>
                    <ArrowLeft />
                  </IconButton>
                </Link>
                <Tooltip title={incomingPeers[i][0].id}>
                  <span>{incomingPeers[i][0].name}</span>
                </Tooltip>
                <MetaDataPreview metaData={incomingPeers[i][0].metaData} />
                <Chip
                  label={
                    typeof incomingPeers[i][1].value === "number"
                      ? incomingPeers[i][1].value
                      : incomingPeers[i][1].value
                      ? `${
                          (incomingPeers[i][1].value as [number, number])[0]
                        } - ${
                          (incomingPeers[i][1].value as [number, number])[1]
                        }`
                      : ""
                  }
                />
              </Item>
            </Grid>
          ) : (
            <Grid item xs={6} />
          )}
          {outgoingPeers[i] ? (
            <Grid item xs={6}>
              <Item
                direction="row"
                alignItems="center"
                justifyContent="space-between"
              >
                <Chip
                  label={
                    typeof outgoingPeers[i][1].value === "number"
                      ? outgoingPeers[i][1].value
                      : outgoingPeers[i][1].value
                      ? `${
                          (outgoingPeers[i][1].value as [number, number])[0]
                        } - ${
                          (outgoingPeers[i][1].value as [number, number])[1]
                        }`
                      : ""
                  }
                />
                <MetaDataPreview metaData={outgoingPeers[i][0].metaData} />
                <Tooltip title={outgoingPeers[i][0].id}>
                  <span>{outgoingPeers[i][0].name}</span>
                </Tooltip>
                <Link
                  to={`/${
                    isTransition(outgoingPeers[i][0]) ? "transition" : "place"
                  }s/${outgoingPeers[i][0].id}`}
                >
                  <IconButton>
                    <ArrowRight />
                  </IconButton>
                </Link>
              </Item>
            </Grid>
          ) : (
            <Grid item xs={6} />
          )}
        </>
      ))}
    </Grid>
  );

  //   return (
  //     <Grid container spacing={1}>
  //       {metaData.map((metaData: MetaData, i: number) => {
  //         const count =
  //           typeof metaData.value === "string"
  //             ? 1
  //             : metaData.value
  //             ? metaData.value.length
  //             : 0;
  //         const values =
  //           typeof metaData.value === "string"
  //             ? [metaData.value]
  //             : metaData.value
  //             ? metaData.value
  //             : [];

  //         const headerSize = count === 0 ? 12 : 3;
  //         const cellSize = count === 0 ? 0 : 9 / count;

  //         return (
  //           <>
  //             <Grid key={`${metaData.type}-md${i}`} item xs={headerSize}>
  //               <Item>
  //                 {metaData.type}
  //               </Item>
  //             </Grid>
  //             {values.map((val, vidx) => (
  //               <Grid key={`${metaData.type}-md${i}-${vidx}`} item xs={cellSize}>
  //                 <Tooltip title={val}>
  //                 <Item color={colorLookup[val]}>
  //                   {nameLookup[val]}
  //                 </Item>

  //                 </Tooltip>
  //               </Grid>
  //             ))}
  //           </>
  //         );
  //       })}
  //     </Grid>
  //   );
}

function isTransition(obj: unknown): obj is Transition {
  return (obj as Transition).cost !== undefined;
}

function isPlace(obj: unknown): obj is Place {
  return (obj as Place).tokens !== undefined;
}
