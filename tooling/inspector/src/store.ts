import net from "../../../output/cost_net.json";
import { mapValues, memoize } from "lodash";
import { interpolateRainbow } from "d3-scale-chromatic";
import { atom } from "jotai";
import { focusAtom } from "jotai-optics";

export type MetaData = {
  type: string;
  value?: (string | number) | (string | number)[];
};

export type Place = {
  id: string;
  name: string;
  tokens: string;
  metaData: MetaData[];
};

export type Signature = {
  type: string;
  value: number | [number, number];
};

export type Transition = {
  id: string;
  name: string;
  metaData: MetaData[];
  input: { [key: string]: Signature };
  output: { [key: string]: Signature };
  time: number;
  cost: number;
};

export type PetriNet = {
  id: string;
  name: string;
  places: { [key: string]: Place };
  transitions: { [key: string]: Transition };
  initialMarking: { [key: string]: number };
  nameLookup: { [key: string]: string };
};

export type MetaDataFilter = {
  type: string | null;
  value?: (string | number | null) | (string | number | null)[];
}

export type Search = {
  name: string | null;
  tags: MetaDataFilter[];
}

export const netAtom = atom<PetriNet>(net);

export const colorLookupAtom = atom<{ [key: string]: string }>((get) => {
  const nameLookup = get(netAtom).nameLookup;
  const colorValues = new Set(Object.values(nameLookup));

  const sizeColor = colorValues.size;
  const valueList = Array.from(colorValues);

  const colorLookup: { [key: string]: string } = mapValues(
    nameLookup,
    (_, key) => interpolateRainbow(valueList.indexOf(nameLookup[key]) / sizeColor)
  );
  return colorLookup;
});

export const placesAtom = focusAtom(netAtom, (optic) => optic.prop("places"));
export const transitionsAtom = focusAtom(netAtom, (optic) =>
  optic.prop("transitions")
);
export const nameLookupAtom = focusAtom(netAtom, (optic) =>
  optic.prop("nameLookup")
);

export const nodeNameLookupAtom = atom<{ [key: string]: string }>((get) => {
  return {
    ...mapValues(get(placesAtom), (place) => place.name),
    ...mapValues(get(transitionsAtom), (transition) => transition.name),
  };
});

export const searchAtom = atom<Search>({name: null, tags: []});

// export const subnetAtom = atom<{
//   [key: string]: { [key: string]: Transition | Place };
// }>((get) => {
//   const net = get(netAtom);
//   console.log("getting subnets");
//   let subnets = {
//     ...mapValues(net.places, (place) => getNeighbors(place.id, net, 0, [])),
//     ...mapValues(net.transitions, (transition) =>
//       getNeighbors(transition.id, net, 0, [])
//     ),
//   };

//   return subnets;
// });

export const directedNeighborsAtom = atom<{ [key: string]:{incoming: Neighbors; outgoing: Neighbors} }>((get) => {
    const net = get(netAtom);
    const directedNeighbors = {
        ...mapValues(net.places, (place) => getDirectedNeighbors(place.id, net)),
        ...mapValues(net.transitions, (transition) =>
            getDirectedNeighbors(transition.id, net)
        ),
        };
    return directedNeighbors;
})

export const markingAtom = focusAtom(netAtom, (optic) =>
  optic.prop("initialMarking")
);

export interface Neighbors {
  [key: string]: [Transition | Place, Signature];
}

// A version of getNeighbors that breaks up incoming from outgoing connections
const getDirectedNeighbors: (
  id: string,
  net: PetriNet
) => { incoming: Neighbors; outgoing: Neighbors } = memoize((id, net) => {
  let incoming: Neighbors = {};
  let outgoing: Neighbors = {};

  if (net.transitions[id]) {
    const transition: Transition = net.transitions[id];

    Object.keys(transition.input).forEach((key) => {
      incoming[key] = [net.places[key], transition.input[key]];
    });
    Object.keys(transition.output).forEach((key) => {
      outgoing[key] = [net.places[key], transition.output[key]];
    });
  } else if (net.places[id]) {
    const neighborTransitions = getPlaceNeighbors(
      id,
      Object.values(net.transitions)
    );

    neighborTransitions.forEach((neighborTransition: Transition) => {
      if (Object.keys(neighborTransition.input).includes(id)) {
        outgoing[neighborTransition.id] = [
          neighborTransition,
          neighborTransition.input[id],
        ];
      } 
      if (Object.keys(neighborTransition.output).includes(id)) {
        incoming[neighborTransition.id] = [
          neighborTransition,
          neighborTransition.output[id],
        ];
      }
    });
  }
  return { incoming, outgoing };
});

const getNeighbors: (
  id: string,
  net: PetriNet,
  boundary: number,
  covered: string[]
) => { [key: string]: Transition | Place } = memoize(
  (id, net, boundary, covered) => {
    let neighbors: { [key: string]: Transition | Place } = {};
    if (net.transitions[id]) {
      const transition: Transition = net.transitions[id];
      let currentCovered = [...covered, id];
      neighbors[id] = transition;

      Object.keys(transition.input).forEach((key) => {
        if (!currentCovered.includes(key)) {
          neighbors[key] = net.places[key];
          if (boundary > 0) {
            const adjacentNeighbors = getNeighbors(
              key,
              net,
              boundary - 1,
              currentCovered
            );
            neighbors = { ...neighbors, ...adjacentNeighbors };
            Object.keys(adjacentNeighbors).forEach((key) => {
              if (!currentCovered.includes(key)) {
                currentCovered.push(key);
              }
            });
          }
        }
      });
      Object.keys(transition.output).forEach((key) => {
        if (!currentCovered.includes(key)) {
          neighbors[key] = net.places[key];
          if (boundary > 0) {
            const adjacentNeighbors = getNeighbors(
              key,
              net,
              boundary - 1,
              currentCovered
            );
            neighbors = { ...neighbors, ...adjacentNeighbors };
            Object.keys(adjacentNeighbors).forEach((key) => {
              if (!currentCovered.includes(key)) {
                currentCovered.push(key);
              }
            });
          }
        }
      });
    } else if (net.places[id]) {
      const neighborTransitions = getPlaceNeighbors(
        id,
        Object.values(net.transitions)
      );
      const place = net.places[id];
      let currentCovered = [...covered, id];
      neighbors[id] = place;

      neighborTransitions.forEach((neighborTransition: Transition) => {
        if (!currentCovered.includes(neighborTransition.id)) {
          neighbors[neighborTransition.id] = neighborTransition;
          if (boundary > 0) {
            const adjacentNeighbors = getNeighbors(
              neighborTransition.id,
              net,
              boundary - 1,
              currentCovered
            );
            neighbors = { ...neighbors, ...adjacentNeighbors };
            Object.keys(adjacentNeighbors).forEach((key) => {
              if (!currentCovered.includes(key)) {
                currentCovered.push(key);
              }
            });
          }
        }
      });
    }
    return neighbors;
  }
);

const getPlaceNeighbors: (
  id: string,
  transitions: Transition[]
) => Transition[] = (id, transitions) => {
  return transitions.filter(
    (transition) =>
      Object.keys(transition.input).includes(id) ||
      Object.keys(transition.output).includes(id)
  );
};

export const arrangementAtom = atom<{ [key: string]: number }>({});
