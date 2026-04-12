import { useCallback, useEffect, useMemo, useState } from "react";
import type { DashboardVariable, ParamValue } from "../gen/open_plx/v1/dashboard_pb.js";
import { create } from "@bufbuild/protobuf";
import { ParamValueSchema, StringListSchema } from "../gen/open_plx/v1/dashboard_pb.js";

/** Variable values keyed by variable name. */
export type VariableValues = Record<string, ParamValue>;

interface UseVariablesResult {
  /** Current variable values. */
  values: VariableValues;
  /** Update a single variable's value. */
  setValue: (name: string, value: ParamValue) => void;
  /** The variable definitions from the dashboard proto. */
  variables: DashboardVariable[];
  /** Revision counter -- increments on every value change to trigger re-fetches. */
  revision: number;
  /** Initialize variable values from URL search params. */
  initFromUrl: () => void;
  /** Sync current variable values to URL search params. */
  syncToUrl: () => void;
}

/** Extract the default ParamValue from a DashboardVariable. */
function getDefaultValue(variable: DashboardVariable): ParamValue | undefined {
  return variable.defaultValue ?? undefined;
}

/**
 * Manages dashboard variable state.
 * Initializes from proto default values, provides current values
 * and a setter that triggers widget re-fetches via revision counter.
 * Re-initializes when the dashboard changes (new variables array).
 *
 * URL sync: variables are persisted to/from URL search params so that
 * dashboard state is shareable via links.
 */
export function useVariables(variables: DashboardVariable[]): UseVariablesResult {
  const defaults = useMemo(() => {
    const map: VariableValues = {};
    for (const v of variables) {
      const def = getDefaultValue(v);
      if (def) {
        map[v.name] = def;
      }
    }
    return map;
  }, [variables]);

  const [values, setValues] = useState<VariableValues>(defaults);
  const [revision, setRevision] = useState(0);

  // Re-initialize when dashboard changes (new defaults computed from new variables).
  useEffect(() => {
    setValues(defaults);
    setRevision(0);
  }, [defaults]);

  const setValue = useCallback((name: string, value: ParamValue) => {
    setValues((prev) => ({ ...prev, [name]: value }));
    setRevision((prev) => prev + 1);
  }, []);

  /** Get the default value for a variable by name. */
  const getDefaultValueByName = useCallback(
    (name: string): ParamValue | undefined => {
      const variable = variables.find((v) => v.name === name);
      return variable ? getDefaultValue(variable) : undefined;
    },
    [variables],
  );

  /**
   * Initialize variable values from URL search params.
   * Converts string URL params to the appropriate ParamValue type
   * based on the current variable's type.
   */
  const initFromUrl = useCallback(() => {
    const params = new URLSearchParams(window.location.search);
    const updates: VariableValues = {};

    for (const [name, value] of params) {
      if (name in values) {
        const currentValue = values[name];
        if (currentValue && typeof currentValue === "object") {
          const case_ = currentValue.value.case;
          if (case_ === "stringValue") {
            updates[name] = create(ParamValueSchema, { value: { case: "stringValue", value } });
          } else if (case_ === "intValue") {
            updates[name] = create(ParamValueSchema, { value: { case: "intValue", value: BigInt(parseInt(value, 10)) } });
          } else if (case_ === "doubleValue") {
            updates[name] = create(ParamValueSchema, { value: { case: "doubleValue", value: parseFloat(value) } });
          } else if (case_ === "boolValue") {
            updates[name] = create(ParamValueSchema, { value: { case: "boolValue", value: value === "true" } });
          } else if (case_ === "stringList") {
            // Multi-select: repeated param values as comma-separated or multiple params
            const listValues = params.getAll(name);
            updates[name] = create(ParamValueSchema, {
              value: { case: "stringList", value: create(StringListSchema, { values: listValues }) },
            });
          } else {
            updates[name] = create(ParamValueSchema, { value: { case: "stringValue", value } });
          }
        } else {
          updates[name] = create(ParamValueSchema, { value: { case: "stringValue", value } });
        }
      }
    }

    if (Object.keys(updates).length > 0) {
      setValues((prev) => ({ ...prev, ...updates }));
    }
  }, [values]);

  /**
   * Sync current variable values to URL search params.
   * Skips values that match their default values to keep URLs clean.
   */
  const syncToUrl = useCallback(() => {
    const params = new URLSearchParams();

    for (const [name, value] of Object.entries(values)) {
      if (value === undefined || value === null) continue;

      // Skip default values -- only sync user-changed values
      const defaultVal = getDefaultValueByName(name);
      if (JSON.stringify(value) === JSON.stringify(defaultVal)) continue;

      // Extract the actual value from ParamValue wrapper
      const case_ = value.value.case;
      if (case_ === "stringValue") {
        params.set(name, value.value.value);
      } else if (case_ === "intValue") {
        params.set(name, String(value.value.value));
      } else if (case_ === "doubleValue") {
        params.set(name, String(value.value.value));
      } else if (case_ === "boolValue") {
        params.set(name, String(value.value.value));
      } else if (case_ === "stringList") {
        // For multi-select, set multiple params with the same key
        for (const v of value.value.value.values) {
          params.append(name, v);
        }
      }
      // Skip variableRef and other complex types
    }

    const newUrl = `${window.location.pathname}?${params.toString()}`;
    window.history.replaceState(null, "", newUrl);
  }, [values, getDefaultValueByName]);

  // Sync to URL on variable change
  useEffect(() => {
    syncToUrl();
  }, [values, syncToUrl]);

  return { values, setValue, variables, revision, initFromUrl, syncToUrl };
}
