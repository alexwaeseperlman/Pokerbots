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
  styled,
} from "@mui/joy";
import KeyboardArrowLeftIcon from "@mui/icons-material/KeyboardArrowLeft";
import KeyboardArrowRightIcon from "@mui/icons-material/KeyboardArrowRight";

export type DataTableData = {};
export type DataTableColumn<T extends DataTableData> = {
  name: string;
  getProps: (row: T) => Record<string, any>;
  render: (props: any) => React.ReactElement;
} & BoxProps &
  React.TdHTMLAttributes<HTMLTableCellElement>;

export interface DataTableProps<T extends DataTableData> {
  data: T[];
  columns: DataTableColumn<T>[];
  onPageChange?: (page: number) => void;
  perPage?: number;
  total?: number;
  serverPagination?: boolean;
  loading: boolean;
}

function shallowCompare(
  a: Record<string, string | number>,
  b: Record<string, string | number>
) {
  if (Object.keys(a).length != Object.keys(b).length) return false;
  for (const key in a) {
    if (a[key] != b[key]) return false;
  }
  return true;
}

let keyCounter = 0;
const keyLookups = new WeakMap<any, number>();

const Cell = styled(
  (props: BoxProps & React.TdHTMLAttributes<HTMLTableCellElement>) => (
    <Box component={(props: any) => <td {...props} />} {...props} />
  )
)(({ theme }) => ({
  whiteSpace: "nowrap",
  overflow: "hidden",
}));

const DataTableCell = React.memo(
  <T extends DataTableData>({
    render: Render,
    props,
  }: {
    render: DataTableColumn<T>["render"];
    props: Record<string, number | string>;
  }) => <Cell>{<Render {...props} />}</Cell>,
  (prev, next) => true
  //prev.render == next.render /*&& shallowCompare(prev.props, next.props)*/
);

const DataTableRow = React.memo(
  <T extends DataTableData>({
    columns,
    row,
  }: {
    columns: DataTableColumn<T>[];
    row: T;
  }) => {
    return (
      <Box component={(props: any) => <tr {...props} />}>
        {columns.map((col, i) => (
          <DataTableCell
            key={col.name}
            render={col.render}
            props={col.getProps(row)}
          />
        ))}
      </Box>
    );
  },
  (prev, next) => {
    const prevProps = prev.columns.map((col) => col.getProps(prev.row));
    const nextProps = next.columns.map((col) => col.getProps(next.row));
    console.log(prevProps, nextProps);
    return (
      prev.columns == next.columns &&
      prevProps.every((prop, i) => shallowCompare(prop, nextProps[i]))
    );
  }
);

export default function DataTable<T extends DataTableData>({
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
      columns.map(({ name, render, getProps, ...props }, i) => (
        <Cell {...props} key={name}>
          {name}
        </Cell>
      )),
    [...columns]
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
          {`${data.length === 0 ? 0 : page * perPage + 1}-${Math.min(
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
    [page, perPage, total, handleChangePage, data.length]
  );

  return (
    <Box>
      <Box>
        <Table {...props}>
          <thead>
            <tr>{...headers}</tr>
          </thead>
          <tbody>
            {pagedData.map((row, i) => (
              <DataTableRow key={i} columns={columns} row={row} />
            ))}
          </tbody>
          <tfoot>
            <tr>
              <Cell colSpan={columns.length}>{paginationControls}</Cell>
            </tr>
          </tfoot>
        </Table>
      </Box>
    </Box>
  );
}
