"""Optional Field Specification."""

from enum import Enum


class OptionalField(str, Enum):
    """Optional Field Specification."""

    required = "yes"
    optional = "optional"
    excluded = "excluded"
