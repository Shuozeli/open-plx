/**
 * Evaluates widget visibility conditions against current variable values.
 * Pure function -- no side effects, no network calls.
 */

import type { VisibilityCondition, ParamValue } from "../gen/open_plx/v1/dashboard_pb.js";
import { ConditionOperator } from "../gen/open_plx/v1/dashboard_pb.js";
import type { VariableValues } from "../hooks/useVariables.js";

/**
 * Returns true if the widget should be visible.
 * All conditions must pass (implicit AND). Empty conditions = always visible.
 */
export function isWidgetVisible(
  conditions: readonly VisibilityCondition[],
  variableValues: VariableValues | undefined,
): boolean {
  if (conditions.length === 0) return true;
  return conditions.every((cond) => evaluateCondition(cond, variableValues));
}

function evaluateCondition(
  cond: VisibilityCondition,
  vars: VariableValues | undefined,
): boolean {
  const paramValue = vars?.[cond.variable];
  const op = cond.operator;

  if (op === ConditionOperator.EMPTY) {
    return isParamEmpty(paramValue);
  }
  if (op === ConditionOperator.NOT_EMPTY) {
    return !isParamEmpty(paramValue);
  }

  const actual = extractScalar(paramValue);
  const expected = extractScalar(cond.value);

  switch (op) {
    case ConditionOperator.EQUALS:
      return actual === expected;
    case ConditionOperator.NOT_EQUALS:
      return actual !== expected;
    case ConditionOperator.GT:
      return toNumber(actual) > toNumber(expected);
    case ConditionOperator.LT:
      return toNumber(actual) < toNumber(expected);
    case ConditionOperator.GTE:
      return toNumber(actual) >= toNumber(expected);
    case ConditionOperator.LTE:
      return toNumber(actual) <= toNumber(expected);
    case ConditionOperator.IN:
      return isValueInList(paramValue, expected);
    default:
      return true; // Unknown operator = visible (safe default)
  }
}

function isParamEmpty(pv: ParamValue | undefined): boolean {
  if (!pv?.value) return true;
  const v = pv.value;
  switch (v.case) {
    case "stringValue":
      return v.value === "";
    case "stringList":
      return v.value.values.length === 0;
    case undefined:
      return true;
    default:
      return false; // numbers, bools, dateRange are never "empty"
  }
}

function extractScalar(pv: ParamValue | undefined): string | number | boolean | undefined {
  if (!pv?.value) return undefined;
  const v = pv.value;
  switch (v.case) {
    case "stringValue":
      return v.value;
    case "intValue":
      return Number(v.value); // bigint -> number for comparison
    case "doubleValue":
      return v.value;
    case "boolValue":
      return v.value;
    default:
      return undefined;
  }
}

function toNumber(v: string | number | boolean | undefined): number {
  if (typeof v === "number") return v;
  if (typeof v === "string") return parseFloat(v) || 0;
  if (typeof v === "boolean") return v ? 1 : 0;
  return 0;
}

/** Check if a scalar expected value exists in a stringList variable. */
function isValueInList(
  pv: ParamValue | undefined,
  expected: string | number | boolean | undefined,
): boolean {
  if (!pv?.value || pv.value.case !== "stringList") return false;
  return pv.value.value.values.includes(String(expected));
}
