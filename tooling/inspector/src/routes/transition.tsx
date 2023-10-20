import { useMemo } from "react";
import { Card, CardHeader, CardContent, Stack } from "@mui/material";
// import "./App.css";
import { useParams } from "react-router-dom";
import { Transition, MetaData } from "../store";
import { colorLookupAtom, transitionsAtom, nameLookupAtom, directedNeighborsAtom, Neighbors } from "../store";
import { selectAtom } from "jotai/utils";
import { focusAtom } from "jotai-optics";
import { atom, useAtomValue } from "jotai";
import MetaDataRenderer from "../MetaDataRenderer";
import PeerRenderer from "../PeerRenderer";

const defaultTransitionAtom = atom<Transition>({
  id: "",
  name: "",
  metaData: [],
  input: {},
  output: {},
  time: 0,
  cost: 0,
});

const defaultNeighborAtom = atom<{ incoming: Neighbors; outgoing: Neighbors }>({
  outgoing: {},
  incoming: {},
});

export default function TransitionCard() {
  const { transitionId } = useParams();
  
  const transitionAtom = useMemo(
    () =>
      transitionId
        ? focusAtom(transitionsAtom, (optic) => optic.prop(transitionId))
        : defaultTransitionAtom,
    [transitionId]
  );
  const transition = useAtomValue(transitionAtom);

  const neighborAtom = useMemo(
    () =>
      transitionId
        ? selectAtom(directedNeighborsAtom, (lookup) => lookup[transitionId])
        : defaultNeighborAtom,
    [transitionId]
  );

  const neighbors = useAtomValue(neighborAtom);

  return (
    <Card key={transition.id}>
      <CardHeader
        title={
          <span>
            <span
              style={{ backgroundColor: "#555", padding: 5, borderRadius: 5 }}
            >
              Transition
            </span>{" "}
            <span
              style={{ backgroundColor: "#333", padding: 5, borderRadius: 5 }}
            >
              {transition.name}
            </span>
          </span>
        }
        action={
          <Stack direction="row" gap={1}>
            <span
              style={{
                backgroundColor: "#666",
                padding: 5,
                borderRadius: 5,
                textTransform: "uppercase",
              }}
            >
              Cost {transition.cost}
            </span>
            <span
              style={{
                backgroundColor: "#666",
                padding: 5,
                borderRadius: 5,
                textTransform: "uppercase",
              }}
            >
              Time {transition.time}
            </span>
          </Stack>
        }
      />
      <CardContent>
        <Stack direction="column" spacing={1}>
          <MetaDataRenderer metaData={transition.metaData} />
          <PeerRenderer neighbors={neighbors} />
        </Stack>
      </CardContent>
    </Card>
  );
}
