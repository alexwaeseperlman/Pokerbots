import * as React from "react";
import Table, { TableProps } from "@mui/joy/Table";
import Sheet from "@mui/joy/Sheet";
import {
  Box,
  BoxProps,
  Divider,
  IconButton,
  Stack,
  Typography,
} from "@mui/joy";
import KeyboardArrowLeftIcon from "@mui/icons-material/KeyboardArrowLeft";
import KeyboardArrowRightIcon from "@mui/icons-material/KeyboardArrowRight";

export interface DataTableProps<T> {
  data: T[];
  columns: ({
    name: string;
    render: (props: { row: T }) => React.ReactElement;
  } & BoxProps)[];
  onPageChange?: (page: number) => void;
  perPage?: number;
  total?: number;
  serverPagination?: boolean;
  loading: boolean;
}

let keyCounter = 0;
const keyLookups = new WeakMap<any, number>();

export default function DataTable<T>({
  data,
  columns,
  onPageChange,
  perPage = Infinity,
  total = data.length,
  serverPagination = false,
  loading = false,
  ...props
}: DataTableProps<T> & TableProps) {
  const headers = React.useMemo(
    () =>
      columns.map(({ name, render, ...props }) => (
        <Box component={(props: any) => <td {...props} />} {...props}>
          {name}
        </Box>
      )),
    [columns]
  );

  const [page, setPage] = React.useState(0);
  const handleChangePage = React.useCallback(
    (page: number) => {
      setPage(page);
      if (onPageChange) onPageChange(page);
    },
    [onPageChange]
  );

  const pagedData = React.useMemo(
    () =>
      serverPagination
        ? data
        : data.slice(page * perPage, page * perPage + perPage),
    [data, perPage, serverPagination, page]
  );

  const rows = React.useMemo(
    () =>
      // This is a hack to get around the fact that we can't use the object as a key
      // in React. We use a WeakMap to store a unique key for each row, and then
      // use that key as the key for the row.
      pagedData.map((row) => {
        let key = keyLookups.get(row);
        if (!key) {
          key = keyCounter++ & ((1 << 52) - 1);
          keyLookups.set(row, key);
        }
        return (
          <>
            <Box key={key} component={(props: any) => <tr {...props} />}>
              {columns.map((col, i) => (
                <td key={i}>{<col.render row={row} />}</td>
              ))}
            </Box>
          </>
        );
      }),
    [pagedData, columns]
  );

  const cards = React.useMemo(
    () =>
      pagedData.map((row) => {
        let key = keyLookups.get(row);
        if (!key) {
          key = keyCounter++ & ((1 << 52) - 1);
          keyLookups.set(row, key);
        }

        return (
          <Stack key={key} gap={1} alignItems="stretch">
            {columns.map((col, i) => (
              <Stack key={i} direction="column" alignItems="stretch">
                <Box>
                  <Typography level={"title-sm"}>{columns[i].name}</Typography>
                </Box>
                {<col.render row={row} />}
              </Stack>
            ))}
          </Stack>
        );
      }),
    [pagedData, columns]
  );

  const paginationControls = React.useMemo(
    () => (
      <Box
        sx={{
          display: "flex",
          alignItems: "center",
          gap: 2,
          justifyContent: "flex-end",
        }}
      >
        <Typography textAlign="center" level="body-sm" sx={{ minWidth: 80 }}>
          {`${rows.length === 0 ? 0 : page * perPage + 1}-${Math.min(
            page * perPage + perPage,
            total
          )} of ${total}`}
        </Typography>
        <Box sx={{ display: "flex", gap: 1, userSelect: "none" }}>
          <IconButton
            size="sm"
            color="neutral"
            variant="outlined"
            disabled={page === 0}
            onClick={() => handleChangePage(page - 1)}
            sx={{ bgcolor: "background.surface" }}
          >
            <KeyboardArrowLeftIcon />
          </IconButton>
          <IconButton
            size="sm"
            color="neutral"
            variant="outlined"
            disabled={
              total !== -1 ? page >= Math.ceil(total / perPage) - 1 : false
            }
            onClick={() => handleChangePage(page + 1)}
            sx={{ bgcolor: "background.surface" }}
          >
            <KeyboardArrowRightIcon />
          </IconButton>
        </Box>
      </Box>
    ),
    [page, perPage, total, handleChangePage, rows.length]
  );

  return (
    <Box>
      <Box
        sx={{
          display: {
            xs: "block",
            sm: "none",
          },
        }}
      >
        <Sheet>
          <Stack gap={4}>
            {cards}
            {paginationControls}
          </Stack>
        </Sheet>
      </Box>
      <Box
        sx={{
          display: {
            xs: "none",
            sm: "block",
          },
        }}
      >
        <Table {...props}>
          <thead>
            <tr>{...headers}</tr>
          </thead>
          <tbody>{...rows}</tbody>
          <tfoot>
            <tr>
              <td colSpan={columns.length}>{paginationControls}</td>
            </tr>
          </tfoot>
        </Table>
      </Box>
    </Box>
  );
}
