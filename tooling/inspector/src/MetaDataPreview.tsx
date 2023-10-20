import { Avatar, AvatarGroup, Paper, Grid, styled, Tooltip } from "@mui/material";
import { MetaData, colorLookupAtom, nameLookupAtom } from "./store";
import { atom, useAtomValue } from "jotai";
// Renders the metadata for a given node as a preview

export default function MetaDataPreview({
  metaData,
}: {
  metaData: MetaData[];
}) {
  const colorLookup = useAtomValue(colorLookupAtom);
  const nameLookup = useAtomValue(nameLookupAtom);

  let uniqueValues: string[] = [];
  metaData.forEach((md: MetaData) => {
    if (md) {
      if (typeof md.value === "string" && !uniqueValues.includes(md.value)) {
        uniqueValues.push(md.value);
      } else if (typeof md.value === "object") {
        md.value.forEach((val: string) => {
          if (!uniqueValues.includes(val)) {
            uniqueValues.push(val)
          }
        });
      }
    }
  })

  return (
    <AvatarGroup max={100}>
      {uniqueValues.map((val: string) => (
        <Tooltip title={nameLookup[val]}>
          <Avatar style={{backgroundColor: colorLookup[val], height:20, width: 20}}>{""}</Avatar>
        </Tooltip>
      ))}

    </AvatarGroup>
  );
}
