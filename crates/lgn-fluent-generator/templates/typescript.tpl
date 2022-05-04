declare type Fluent =
  {%- for description in entry_descriptions %}
  | {
      id: "{{ description.id }}";
      {%- if description.attributes.len() > 0 %}
      attributes: (
        {%- for attribute in description.attributes %}
        | "{{ attribute }}"
        {%- endfor %}
      )[];
      {%- else %}
      attributes: [];
      {%- endif %}
      {%- if description.variables.len() > 0 %}
      variables: (
        {%- for variable in description.variables %}
        | "{{ variable }}"
        {%- endfor %}
      )[];
      {%- else %}
      variables: [];
      {%- endif %}
    }
  {%- endfor %};
