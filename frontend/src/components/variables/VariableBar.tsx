import { Flex, Typography } from "antd";
import type { DashboardVariable, ParamValue } from "../../gen/open_plx/v1/dashboard_pb.js";
import type { VariableValues } from "../../hooks/useVariables.js";
import { VariableControl } from "./VariableControl.js";

interface VariableBarProps {
  variables: DashboardVariable[];
  values: VariableValues;
  onValueChange: (name: string, value: ParamValue) => void;
}

/**
 * Horizontal bar of variable controls rendered above the dashboard grid.
 * Each variable maps to an Antd input component based on its control type.
 */
export function VariableBar({ variables, values, onValueChange }: VariableBarProps) {
  if (variables.length === 0) return null;

  return (
    <Flex gap={16} align="center" wrap="wrap" style={{ marginBottom: 16 }}>
      {variables.map((variable) => (
        <Flex key={variable.name} gap={4} align="center">
          <Typography.Text strong style={{ whiteSpace: "nowrap" }}>
            {variable.label}:
          </Typography.Text>
          <VariableControl
            variable={variable}
            value={values[variable.name]}
            onChange={(val) => onValueChange(variable.name, val)}
          />
        </Flex>
      ))}
    </Flex>
  );
}
