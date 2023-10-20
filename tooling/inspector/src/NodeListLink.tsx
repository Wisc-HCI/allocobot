import {
  ListItemButton,
  ListItemIcon
} from "@mui/material";
import { useParams } from "react-router-dom";
import { nodeNameLookupAtom } from "./store";
import RadioButtonUncheckedIcon from "@mui/icons-material/RadioButtonUnchecked";
import CheckBoxOutlineBlankIcon from "@mui/icons-material/CheckBoxOutlineBlank";
import { NavLink } from "react-router-dom";
import { useAtomValue } from "jotai";
import { TextInput } from "./TextInput";

export interface NodeListLinkProps {
  id: string;
  type: "place" | "transition";
}
export default function NodeListLink({ id, type }: NodeListLinkProps) {

  const nodeNameLookup = useAtomValue(nodeNameLookupAtom);
  const params = useParams();

  return (
    <NavLink
      to={`/${type}s/${id}`}
      key={id}
      style={{ all: "unset" }}
    >
      <ListItemButton key={id} selected={id === params.placeId || id === params.transitionId}>
        <ListItemIcon>
          {type === 'place' ? <RadioButtonUncheckedIcon /> : <CheckBoxOutlineBlankIcon />}
        </ListItemIcon>
        <TextInput readonly value={nodeNameLookup[id]} />
      </ListItemButton>
    </NavLink>
  );
}
