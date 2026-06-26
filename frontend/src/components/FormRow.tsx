import type { ReactNode } from "react";

type FormRowProps = {
  label: string;
  children: ReactNode;
};

export function FormRow({ label, children }: FormRowProps) {
  return (
    <tr>
      <th scope="row">{label}</th>
      <td>{children}</td>
    </tr>
  );
}
