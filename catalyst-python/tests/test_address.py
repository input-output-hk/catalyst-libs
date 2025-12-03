# ruff: noqa: S101, ERA001, D100, D103

from catalyst_python import address


def test_bech32_encoding() -> None:
    addr = address.stake_public_key_to_address(
        # cspell: disable-next-line
        key="7d84a4ac0a98a10f92b8a11e76a5d33e5400a0ea77f0826f789fcb37db6365fb",
        is_stake=True,
        network_type="testnet",
    )
    # cspell: disable-next-line
    assert addr == "stake_test1url2pfd6d6dlfy7z880hxhgae7gflh2tgyr8y34weu0y2qq5qcm8q"

    addr = address.stake_public_key_to_address(
        # cspell: disable-next-line
        key="2ff0e0382ef9f3a15b8331b6c417ee899641c3fc43bd45e2ec7915b27c8989f5",
        is_stake=True,
        network_type="testnet",
    )
    # cspell: disable-next-line
    assert addr == "stake_test1uztzn5dgv07qvjqv23lua6pnz3q5tyz3prxnk7sz0dvt65qjf8t05"
