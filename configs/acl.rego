# Role-based Access Control (RBAC)
# --------------------------------
#
# This example defines an RBAC model for a Pet Store API. The Pet Store API allows
# users to look at pets, adopt them, update their stats, and so on. The policy
# controls which users can perform actions on which resources. The policy implements
# a classic Role-based Access Control model where users are assigned to roles and
# roles are granted the ability to perform some action(s) on some type of resource.
#
# This example shows how to:
#
#	* Define an RBAC model in Rego that interprets role mappings represented in JSON.
#	* Iterate/search across JSON data structures (e.g., role mappings)
#
# For more information see:
#
#	* Rego comparison to other systems: https://www.openpolicyagent.org/docs/latest/comparison-to-other-systems/
#	* Rego Iteration: https://www.openpolicyagent.org/docs/latest/#iteration

package app.rbac
# By default, deny requests.

main {
    # set the role  if the ressource id is present in the role attribute of the user
    role = roles_attributes[_].value

    # set the role attribute of the user by the resource name (eval input)
    roles_attributes = data.roles[input.eval]
    # check if the ressource id is present in the role attribute of the user
    roles_attributes[_].key == input.resource
    role = roles_attributes[_].value
    matchUrl with input as {
       "eval": input.eval,
       "path": input.uri,
       "method": input.method,
       "role": role
    }
}

matchUrl {
api_attributes =
    [{
      "get": {
        { "key":"api/projects", "value": ["admin", "owner", "billing", "editor", "contributor"] },
        { "key":"api/project/{project_id:[0-9]+}", "value": ["admin", "owner", "billing", "editor", "contributor"] },
        { "key":"api/projects/stacks", "value": ["admin", "owner", "billing", "editor",  "contributor"] }
        },
      "post": {
        { "key":"api/project", "value": ["admin", "owner"] }
       },
      "delete": {
        { "key":"api/project/{project_id:[0-9]+}", "value":["admin"] }
       },
      "put": {
        { "key":"api/project", "value":["admin"] }
       }
    }]

api_resources =
   [
      "private_projects",
      "affiliate_projects",
      "organizations",
      "scopes"
   ]

    api_resources[_] == input.eval
    uri_list = api_attributes[_][input.method]
    uri_list[_].key == input.path
    uri_list[_].value[_] == input.role
}
