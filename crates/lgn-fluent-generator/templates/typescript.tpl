/** A full dictionnary of all the found message ids, attributes, and variables */
declare type Fluent = {
  {%- for description in entry_descriptions %}
  "{{ description.id }}": {
    {%- if description.attributes.len() > 0 %}
    attributes: (
      {%- for attribute in description.attributes %}
      | "{{ attribute }}"
      {%- endfor %}
    );
    {%- else %}
    attributes: null;
    {%- endif %}
    {%- if description.variables.len() > 0 %}
    variables: (
      {%- for variable in description.variables %}
      | "{{ variable }}"
      {%- endfor %}
    );
    {%- else %}
    variables: null;
    {%- endif %}
  };
  {% endfor -%}
};
