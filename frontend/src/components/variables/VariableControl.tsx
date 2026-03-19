import { Cascader, DatePicker, Input, InputNumber, Select } from "antd";
import type { DashboardVariable, ParamValue } from "../../gen/open_plx/v1/dashboard_pb.js";
import { create } from "@bufbuild/protobuf";
import { ParamValueSchema, StringListSchema } from "../../gen/open_plx/v1/dashboard_pb.js";
import dayjs from "dayjs";

interface VariableControlProps {
  variable: DashboardVariable;
  value: ParamValue | undefined;
  onChange: (value: ParamValue) => void;
}

/** Extract current string value from a ParamValue. */
function getStringValue(value: ParamValue | undefined): string {
  if (!value) return "";
  if (value.value.case === "stringValue") return value.value.value;
  return "";
}

/** Extract current number value from a ParamValue. */
function getNumberValue(value: ParamValue | undefined): number | null {
  if (!value) return null;
  if (value.value.case === "intValue") return Number(value.value.value);
  if (value.value.case === "doubleValue") return value.value.value;
  return null;
}

/** Extract string list from a ParamValue. */
function getStringListValue(value: ParamValue | undefined): string[] {
  if (!value) return [];
  if (value.value.case === "stringList") return value.value.value.values;
  return [];
}

/** Create a ParamValue with a string. */
function stringParam(s: string): ParamValue {
  return create(ParamValueSchema, { value: { case: "stringValue", value: s } });
}

/** Create a ParamValue with a number. */
function numberParam(n: number): ParamValue {
  return create(ParamValueSchema, { value: { case: "doubleValue", value: n } });
}

/** Create a ParamValue with a string list. */
function stringListParam(values: string[]): ParamValue {
  return create(ParamValueSchema, {
    value: {
      case: "stringList",
      value: create(StringListSchema, { values }),
    },
  });
}

/**
 * Renders the appropriate Antd control for a dashboard variable.
 * Maps the proto control oneof to an Antd component.
 */
export function VariableControl({ variable, value, onChange }: VariableControlProps) {
  const control = variable.control;

  switch (control.case) {
    case "textInput": {
      const ctrl = control.value;
      return (
        <Input
          value={getStringValue(value)}
          placeholder={ctrl.placeholder || undefined}
          maxLength={ctrl.maxLength || undefined}
          onChange={(e) => onChange(stringParam(e.target.value))}
          style={{ width: 200 }}
        />
      );
    }

    case "numberInput": {
      const ctrl = control.value;
      return (
        <InputNumber
          value={getNumberValue(value)}
          placeholder={ctrl.placeholder || undefined}
          min={ctrl.min ?? undefined}
          max={ctrl.max ?? undefined}
          step={ctrl.step ?? undefined}
          onChange={(val) => {
            if (val !== null) onChange(numberParam(val));
          }}
          style={{ width: 150 }}
        />
      );
    }

    case "select": {
      const ctrl = control.value;
      return (
        <Select
          value={getStringValue(value) || undefined}
          placeholder={ctrl.placeholder || undefined}
          allowClear={ctrl.allowClear}
          showSearch={ctrl.showSearch}
          options={ctrl.options.map((o) => ({ value: o.value, label: o.label }))}
          onChange={(val) => onChange(stringParam(val ?? ""))}
          style={{ minWidth: 150 }}
        />
      );
    }

    case "multiSelect": {
      const ctrl = control.value;
      return (
        <Select
          mode="multiple"
          value={getStringListValue(value)}
          placeholder={ctrl.placeholder || undefined}
          maxCount={ctrl.maxSelections || undefined}
          options={ctrl.options.map((o) => ({ value: o.value, label: o.label }))}
          onChange={(vals) => onChange(stringListParam(vals))}
          style={{ minWidth: 200 }}
        />
      );
    }

    case "datePicker": {
      const strVal = getStringValue(value);
      return (
        <DatePicker
          value={strVal ? dayjs(strVal) : null}
          onChange={(date) => {
            if (date) onChange(stringParam(date.format("YYYY-MM-DD")));
          }}
        />
      );
    }

    case "dateRange": {
      // TODO(refactor): Handle date range presets
      return (
        <DatePicker.RangePicker
          onChange={(dates) => {
            if (dates && dates[0] && dates[1]) {
              onChange(
                create(ParamValueSchema, {
                  value: {
                    case: "dateRange",
                    value: {
                      $typeName: "open_plx.v1.DateRange",
                      start: dates[0].format("YYYY-MM-DD"),
                      end: dates[1].format("YYYY-MM-DD"),
                    },
                  },
                }),
              );
            }
          }}
        />
      );
    }

    case "cascader": {
      const ctrl = control.value;
      interface CascaderOpt {
        value: string;
        label: string;
        children?: CascaderOpt[];
      }
      const mapOptions = (opts: typeof ctrl.options): CascaderOpt[] =>
        opts.map((o) => ({
          value: o.value,
          label: o.label,
          children: o.children.length > 0 ? mapOptions(o.children) : undefined,
        }));
      return (
        <Cascader
          options={mapOptions(ctrl.options)}
          placeholder={ctrl.placeholder || undefined}
          onChange={(val) => {
            if (val && val.length > 0) {
              onChange(stringParam(String(val[val.length - 1])));
            }
          }}
          style={{ minWidth: 200 }}
        />
      );
    }

    default:
      return <span>Unknown control type</span>;
  }
}
