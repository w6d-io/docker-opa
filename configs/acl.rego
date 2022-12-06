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
import future.keywords.if

default main = false

main {
	some i

	#    # set the role  if the ressource id is present in the role attribute of the user
	roles = data.metadata_admin[_]
	roles[i].key == input.resource
	role := roles[i].value
	matchUrl with input as {
		"method": input.method,
		"uri": input.uri,
		"role": role,
	}
}

#transform uri query to base query
uri := "api/projects" if{
    	regex.match(`api\/projects\?projectId=[0-9]+`, input.uri)
} else := input.uri

# check end point right
matchUrl {
	some k
	api_attributes = {
		"get": [
			{"key": "api/projects", "value": ["admin", "owner", "billing", "editor", "contributor"]},
			{"key": "api/projects/stacks", "value": ["admin", "owner", "billing", "editor", "contributor"]},
		],
		"post": {{"key": "api/projects", "value": ["admin", "owner"]}},
		"put": {{"key": "api/projects", "value": ["admin"]}},
	}

	uri_list := api_attributes[input.method]
	uri_list[k].key == uri
	uri_list[k].value[_] == input.role
}
