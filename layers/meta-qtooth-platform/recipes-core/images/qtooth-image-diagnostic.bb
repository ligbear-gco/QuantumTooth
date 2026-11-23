# @file
# @brief Pi-only diagnostic extension; the base image remains free of test workloads.
require qtooth-image-base.bb
SUMMARY = "QuantumTooth Pi platform with bounded diagnostic workloads"
COMPATIBLE_MACHINE = "^raspberrypi4-64$"
IMAGE_INSTALL:append = " rt-tests stress-ng procps-ps util-linux-chrt qtooth-physical-access"

# @brief Permit diagnostic key authentication while keeping passwords unusable.
inherit extrausers
EXTRA_USERS_PARAMS = "usermod -p '*' root;"
