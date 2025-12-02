version: "1.0.0"
project:
{
    name: "libs-docs"

    release: {
		docs: {
			on: {
				merge: {}
				pr: {}
			}

			config: {
				name: "libs"
			}
		}
	}
}
