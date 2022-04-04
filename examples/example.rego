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
default main = false

main {
  input.eval == "organizations"
  organizations

  input.eval == "private_projects"
  private_projects

  input.eval == "scopes"
  scopes

  input.eval == "affiliate_projects"
  affiliate_projects
}

organizations {
    data.roles.organizations[_].key == input.resource
    data.roles.organizations[_].value == input.role
}

private_projects {
    data.roles.private_projects[_].key == input.resource
    data.roles.private_projects[_].value == input.role
}

scopes {
    data.roles.scopes[_].key == input.resource
    data.roles.scopes[_].value == input.role
}

affiliate_projects {
    data.roles.affiliate_projects[_].key == input.resource
    data.roles.affiliate_projects[_].value == input.role
}

