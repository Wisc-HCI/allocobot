import { Paper, Grid, styled, Tooltip } from "@mui/material";
import { MetaData, colorLookupAtom, nameLookupAtom } from "./store";
import { atom, useAtomValue } from "jotai";
// Renders the metadata for a given node as a table

interface ItemProps {
    color?: string;
}
const Item = styled(Paper)<ItemProps>(({ theme, color }) => ({
    backgroundColor: color ? color : theme.palette.mode === 'dark' ? "#555" : '#bbb',
    ...theme.typography.body2,
    padding: theme.spacing(1),
    textAlign: 'center',
    color: theme.palette.text.secondary,
  }));

export default function MetaDataRenderer({
  metaData,
}: {
  metaData: MetaData[];
}) {
  const colorLookup = useAtomValue(colorLookupAtom);
  const nameLookup = useAtomValue(nameLookupAtom);

  return (
    <Grid container spacing={1}>
      {metaData.map((metaData: MetaData, i: number) => {
        const count =
          typeof metaData.value === "string"
            ? 1
            : typeof metaData.value === "number" 
            ? 1
            : metaData.value
            ? metaData.value.length
            : 0;
        const values: (number | string)[] =
          typeof metaData.value === "string" || typeof metaData.value === "number"
            ? [metaData.value]
            : metaData.value
            ? metaData.value
            : [];

        const headerSize = count === 0 ? 12 : 3;
        const cellSize = count === 0 ? 0 : 9 / count;

        return (
          <>
            <Grid key={`${metaData.type}-md${i}`} item xs={headerSize}>
              <Item>
                {metaData.type}
              </Item>
            </Grid>
            {values.map((val, vidx) => (
              <Grid key={`${metaData.type}-md${i}-${vidx}`} item xs={cellSize}>
                <Tooltip title={val}>
                <Item color={typeof val === 'number' ? undefined : colorLookup[val]}>
                  {typeof val === 'number' ? val : nameLookup[val]}
                </Item>

                </Tooltip>
              </Grid>
            ))}
          </>
        );
      })}
    </Grid>
  );
}
