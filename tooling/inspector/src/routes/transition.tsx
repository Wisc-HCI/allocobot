import { useMemo } from "react";
import { Card, CardHeader, CardContent, Stack, Tooltip } from "@mui/material";
// import "./App.css";
import { useParams } from "react-router-dom";
import { Transition, Cost } from "../store";
import { transitionsAtom, directedNeighborsAtom, Neighbors } from "../store";
import { selectAtom } from "jotai/utils";
import { focusAtom } from "jotai-optics";
import { atom, useAtomValue } from "jotai";
import MetaDataRenderer from "../MetaDataRenderer";
import PeerRenderer from "../PeerRenderer";
import { BackHand, LooksOne, Repeat, Sell } from "@mui/icons-material";

const defaultTransitionAtom = atom<Transition>({
  id: "",
  name: "",
  metaData: [],
  input: {},
  output: {},
  time: 0,
  cost: [],
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
    [transitionId],
  );
  const transition = useAtomValue(transitionAtom);

  const neighborAtom = useMemo(
    () =>
      transitionId
        ? selectAtom(directedNeighborsAtom, (lookup) => lookup[transitionId])
        : defaultNeighborAtom,
    [transitionId],
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
            {transition.cost.map((cost: Cost) => (
              <Stack
                direction="row"
                gap={1}
                alignItems="center"
                style={{
                  backgroundColor: "#666",
                  padding: 5,
                  borderRadius: 5,
                  textTransform: "uppercase",
                }}
              >
                <Tooltip title={cost.category.toUpperCase()}>
                  {CategoryIcon[cost.category]}
                </Tooltip>
                <Tooltip title={cost.frequency.toUpperCase()}>
                  {FrequencyIcon[cost.frequency]}
                </Tooltip>
                {cost.value.toFixed(4)}
              </Stack>
            ))}
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

const CategoryIcon = {
  monetary: <Sell sx={{ fontSize: 14 }} />,
  ergonomic: <BackHand sx={{ fontSize: 14 }} />,
};

const FrequencyIcon = {
  once: <LooksOne sx={{ fontSize: 14 }} />,
  perTime: <Repeat sx={{ fontSize: 14 }} />,
};
