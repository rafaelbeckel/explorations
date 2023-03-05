// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

contract ZombieFactory {
    uint256 dnaDigits = 16;
    uint256 dnaModulus = 10**dnaDigits;

    struct Zombie {
        string name;
        uint256 dna;
    }

    Zombie[] public zombies;

    function createZombie(string memory _name, uint256 _dna) public {
        zombies.push(Zombie(_name, _dna));
    }
}
