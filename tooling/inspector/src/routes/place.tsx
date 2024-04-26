import { useMemo } from "react";
import { Card, CardHeader, CardContent, Stack } from "@mui/material";
// import "./App.css";
import { useParams } from "react-router-dom";
import {
  Place,
  directedNeighborsAtom,
  Neighbors,
  placesAtom,
  markingAtom,
} from "../store";
import { focusAtom } from "jotai-optics";
import { atom, useAtomValue } from "jotai";
import { selectAtom } from "jotai/utils";
import MetaDataRenderer from "../MetaDataRenderer";
import PeerRenderer from "../PeerRenderer";

const defaultPlaceAtom = atom<Place>({
  id: "",
  name: "",
  tokens: "",
  metaData: [],
});

const defaultNeighborAtom = atom<{ incoming: Neighbors; outgoing: Neighbors }>({
  outgoing: {},
  incoming: {},
});

export default function PlaceCard() {
  const { placeId } = useParams();

  const placeAtom = useMemo(
    () =>
      placeId
        ? focusAtom(placesAtom, (optic) => optic.prop(placeId))
        : defaultPlaceAtom,
    [placeId],
  );
  const place = useAtomValue(placeAtom);

  const neighborAtom = useMemo(
    () =>
      placeId
        ? selectAtom(directedNeighborsAtom, (lookup) => lookup[placeId])
        : defaultNeighborAtom,
    [placeId],
  );

  const neighbors = useAtomValue(neighborAtom);

  const marking = useAtomValue(markingAtom);

  return (
    <Card key={place.id}>
      <CardHeader
        title={
          <span>
            <span
              style={{ backgroundColor: "#555", padding: 5, borderRadius: 5 }}
            >
              Place
            </span>{" "}
            <span
              style={{ backgroundColor: "#333", padding: 5, borderRadius: 5 }}
            >
              {place.name}
            </span>
          </span>
        }
        action={
          <span
            style={{
              backgroundColor: "#666",
              padding: 5,
              borderRadius: 5,
              textTransform: "uppercase",
            }}
          >
            {place.tokens}
            {marking[place.id] ? ` (${marking[place.id]})` : ""}
          </span>
        }
      />
      <CardContent>
        <Stack direction="column" spacing={1}>
          <MetaDataRenderer metaData={place.metaData} />
          <PeerRenderer neighbors={neighbors} />
        </Stack>
      </CardContent>
    </Card>
  );
}
