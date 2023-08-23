import * as React from "react";
import Table, { TableProps } from "@mui/joy/Table";
import Sheet from "@mui/joy/Sheet";
import { Box, IconButton, Typography } from "@mui/joy";
import KeyboardArrowLeftIcon from "@mui/icons-material/KeyboardArrowLeft";
import KeyboardArrowRightIcon from "@mui/icons-material/KeyboardArrowRight";

export interface DataTableProps<T> {
  data: T[];
  columns: {
    name: string;
    render: (row: T) => React.ReactNode;
  }[];
  onPageChange?: (page: number) => void;
  perPage?: number;
  total?: number;
  serverPagination?: boolean;
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
  ...props
}: DataTableProps<T> & TableProps) {
  const headers = React.useMemo(
    () => columns.map((col) => <th>{col.name}</th>),
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
          <tr key={key.toString()}>
            {columns.map((col, i) => (
              <td key={i}>{col.render(row)}</td>
            ))}
          </tr>
        );
      }),
    [pagedData, columns]
  );

  return (
    <Table {...props}>
      <thead>{...headers}</thead>
      <tbody>{...rows}</tbody>
      <tfoot>
        <tr>
          <td colSpan={columns.length}>
            <Box
              sx={{
                display: "flex",
                alignItems: "center",
                gap: 2,
                justifyContent: "flex-end",
              }}
            >
              <Typography
                textAlign="center"
                level="body-sm"
                sx={{ minWidth: 80 }}
              >
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
                    total !== -1
                      ? page >= Math.ceil(total / perPage) - 1
                      : false
                  }
                  onClick={() => handleChangePage(page + 1)}
                  sx={{ bgcolor: "background.surface" }}
                >
                  <KeyboardArrowRightIcon />
                </IconButton>
              </Box>
            </Box>
          </td>
        </tr>
      </tfoot>
    </Table>
  );
}
