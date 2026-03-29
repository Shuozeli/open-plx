import { useCallback, useEffect, useMemo, useState } from "react";
import type { DashboardVariable, ParamValue } from "../gen/open_plx/v1/dashboard_pb.js";

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

  return { values, setValue, variables, revision };
}
