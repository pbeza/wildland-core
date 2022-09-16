require 'json'
require 'semantic'

changed = `cargo workspaces changed --json --include-merged-tags`.chomp

# Exit code 42 is a conditional CI failure

# This may happen if there were no crates changes yet we are in the same HEAD (rare occasions when re-running the pipeline)
changed == 'Current HEAD is already released, skipping change detection' and exit 42

changed = JSON.parse(changed)

# There are no changes, yet we are running this pipeline, perhaps we didn't touch any crates.
# Still, better throw 42 to grab the reviewer's attention.
changed.size == 0 and exit 42

changed.each { |e|
	crate, current_version = e['name'], e['version']

	puts "Checking crate: #{crate} v[#{current_version}]"

	search_result = `cargo search --registry=wl-dev #{crate}`

	# If there were no results from search, it means the package is not published there yet, thus
	# we can move forward with any version
	search_result.empty? and next

	registry_version = search_result.scan(/#{crate} = "(.+?)"/)

	# Couldn't get crate version from the registry
	registry_version.empty? and exit 2

	current_version = Semantic::Version.new current_version
	registry_version = Semantic::Version.new registry_version.first.first

	# If the current (ie new) version it not newer, exit with code 3
	current_version > registry_version or exit 3
}
