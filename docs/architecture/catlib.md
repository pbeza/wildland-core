# Catlib

## Definition
Catlib (also referred to as catalog backend) represents a decentralized database that serves as synchronization endpoint for multiple devices. The goal of the component is to keep the Wildland Core's data in sync among devices using the same containers, forests, storages and bridges.

## Catlib Storage
Catlib implements CoreX interfaces for the objects for the following objects:
* `ForestManifest`
* `ContainerManifest`
* `StorageManifest`
* `BridgeManifest`
* `CatlibService`

The goal of Catlib is to orginise the data structures in a form that can be used in some underlying database engine. Currently used database is `redis`.

## Catlib in relate to other components
Catlib instance is created in Cargolib component and is immediately passed to CoreX (it's not used by Cargolib itself). CoreX stores it and uses it to synchronize the data related to objects listed above. The type of the database used in Catlib is transparent for CoreX.

## Manifests
```mermaid
flowchart TB
UserA(Wildland User A):::personType
subgraph UserADevices[User A Devices]
    Device1[Device 1]:::deviceType
    Device2[Device 2]:::deviceType
end
UserB(Wildland User B):::personType
subgraph UserBDevices[User B Devices]
    Device3[Device 3]:::deviceType
    Device4[Device 4]:::deviceType
end
subgraph forestSpace1[Forest 1]
    subgraph ForestManifest1[Forest Manifest 1]
        forestDescription1[ - public keys of user devices \n - list of containers own by the forest]:::descriptionType
    end
    ForestManifest1:::forestManifestType
    subgraph ContainerManifest1[Container Manifest 1]
        containerDescription1[ - container paths \n - storages manifests \n - data encryption key]:::descriptionType
    end
    ContainerManifest1:::containerManifestType
    subgraph ContainerManifest2[Container Manifest 2]
        containerDescription2[ - container paths \n - storages manifests \n - data encryption key]:::descriptionType
    end
    ContainerManifest2:::containerManifestType
    subgraph StorageManifest1[Storage Manifest 1]
        storageDescription1[ - data backend related credentials]:::descriptionType
    end
    StorageManifest1:::storageManifestType
    subgraph StorageManifest2[Storage Manifest 2]
        storageDescription2[ - data backend related credentials]:::descriptionType
    end
    StorageManifest2:::storageManifestType
    subgraph BridgeManifest1[Bridge Manifest 1]
        bridgeDescription1[ - wildland URL to another forest ]:::descriptionType
    end
    BridgeManifest1:::bridgeManifestType
end
subgraph forestSpace2[Forest 2]
    subgraph ForestManifest2[Forest Manifest 2]
        forestDescription2[ - public keys of user devices\n - list of containers own by the forest]:::descriptionType
    end
    ForestManifest2:::forestManifestType
    subgraph ContainerManifest3[Container Manifest 3]
        containerDescription3[ - container paths\n - storages manifests \n - data encryption key]:::descriptionType
    end
    ContainerManifest3:::containerManifestType
    subgraph ContainerManifest4[Container Manifest 4]
        containerDescription4[ - container paths\n - storages manifests \n - data encryption key]:::descriptionType
    end
    ContainerManifest4:::containerManifestType
    subgraph StorageManifest3[Storage Manifest 3]
        storageDescription3[ - data backend related credentials]:::descriptionType
    end
    StorageManifest3:::storageManifestType
    subgraph StorageManifest4[Storage Manifest 4]
        storageDescription4[ - data backend related credentials]:::descriptionType
    end
    StorageManifest4:::storageManifestType
end
classDef descriptionType stroke-width:0px, color:#000000, fill:transparent
classDef personType fill:#4c97f3
classDef forestManifestType fill:#4ec241
classDef storageManifestType fill:#ebe9c1
classDef containerManifestType fill:#89d5f4
classDef bridgeManifestType fill:#cb7d2c
classDef deviceType fill:#d19b0d
UserA --> Device1
UserA --> Device2
UserB --> Device3
UserB --> Device4
Device1 --> ForestManifest1
Device2 --> ForestManifest1
Device3 --> ForestManifest2
Device4 --> ForestManifest2
ForestManifest1 --> ContainerManifest1
ForestManifest1 --> ContainerManifest2
ForestManifest1 --> BridgeManifest1
ForestManifest2 --> ContainerManifest3
ForestManifest2 --> ContainerManifest4
BridgeManifest1 -----> ForestManifest2
ContainerManifest1 --> StorageManifest1
ContainerManifest2 --> StorageManifest2
ContainerManifest3 --> StorageManifest3
ContainerManifest4 --> StorageManifest4
```

## Integrity of manifests
The owner of the forest (that implies being owner of the all manifests belonging to the Forest entity in the network) is responsible and authorized to provide integrity of the forest's manifests. It means that the manifests can be changed only by the owner and should be signed with his/her private key. Other users may gain the READ-ONLY access to the data storage of the manifests and should be able to verify that the given manifests were modified by the owner.

 * **Open question:** What instance should play the role of the Trusted 3rd Party?


## Off-line mode and conflicts from Catlib perspective
Onboarded user should be able to read the last data stored in Catlib cache on the device that is off-line. It should also be possible to modify the local Catlib state off-line (for instance to create a new container, modify paths etc.). If a Catlib's state conflict occurs after going online, user should be able to decide which version should be considered the current one (local state of containers or the already uploaded one).

Conflict in Catlib state may occur only in a situation where a user uses at least two devices:
 * Two devices provides changes to the same manifest at the same time
 * One of the device goes off-line and user makes some changes on it without syncing for some time. In the meantime the same user modifies state on some other device. After goin


 * **Open question:** Is there any better conflict-resolution strategy for Catlib state?

## Manifests encryption
Currently (3rd of Feb 2023) the data is **not encrypted** in Catlib. Although the plan for this component is to encrypt the data transparently for the Catlib user (providing the user dencryption key required).


It's not always required to keep all the manifests encrypted. Though when encryption mode is enabled, only the owner of the forest should be able to decrypt the _Forest Manifest_. Rest of the manifests should be encrypted with different keys. This is important for the containers sharing concept. Example:
 * Three users: _UserA_, _UserB_, _UserC_,
 * _UserA_ is the owner of the _Forest_ that contains 3 containers: _C1_, _C2_, _C3_,
 * _UserA_ lets _UserB_ to decrypt _C2_ manifest, but keeps _C1_ and _C3_ private.
 * _UserA_ lets _UserC_ to decrypt _C3_ manifest, but keeps _C1_ and _C2_ private.

In order to achieve such a goal, each container manifest and related to the given container manifests should be encrypted with different keys.

 * **Open question:** What type of encryption method to use?
 * **Open question:** Where the encryption keys should be stored?
