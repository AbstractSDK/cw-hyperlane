# Hyperlane Configuration Reference

This document provides a reference for the `deploy` section of `config.yaml`, specifically for configuring Interchain Security Modules (ISMs) and Hooks.

## Interchain Security Modules (ISMs)

ISMs are responsible for verifying the authenticity of interchain messages. You configure them under the `ism` key in the `deploy` section.

### Multisig ISM (`multisig`)

*   **Description:** Verifies messages that are signed by a quorum of validators. The validator set and threshold are defined for each source chain.
*   **When to use:** This is the most common and recommended ISM for production. It provides a configurable level of security.
*   **Example:**
    ```yaml
    ism:
      type: multisig
      owner: <signer>
      validators:
        # Domain ID of the source chain
        44787: # e.g., Alfajores
          addrs:
            - '0xValidator1Address...'
            - '0xValidator2Address...'
          threshold: 2
    ```

### Routing ISM (`routing`)

*   **Description:** Routes verification to different ISMs based on the origin domain of the message.
*   **When to use:** When you are connecting to multiple chains and want to use different security models for each.
*   **Example:**
    ```yaml
    ism:
      type: routing
      owner: <signer>
      isms:
        # Route messages from Alfajores (44787) to a multisig ISM
        44787:
          type: multisig
          owner: <signer>
          validators:
            addrs: ['<signer>']
            threshold: 1
        # Route messages from Sepolia (11155111) to a mock ISM for testing
        11155111:
          type: mock
    ```

### Aggregate ISM (`aggregate`)

*   **Description:** Combines multiple ISMs. A message is considered verified if **any** of the aggregated ISMs successfully verifies it.
*   **When to use:** When you want to provide multiple security paths for verification, for example, allowing a message to be verified by either a multisig set or a proof-of-authority ISM.
*   **Example:**
    ```yaml
    ism:
      type: aggregate
      owner: <signer>
      isms:
        - type: multisig
          owner: <signer>
          validators:
            44787:
              addrs: ['<signer>']
              threshold: 1
        - type: mock # A second ISM
    ```

### Pausable ISM (`pausable`)

*   **Description:** A wrapper around another ISM that allows an owner to pause message verification.
*   **When to use:** As a security measure to halt message processing in case of an emergency.
*   **Example:**
    ```yaml
    ism:
      type: pausable
      owner: <signer>
      paused: false
      # The inner ISM to be wrapped
      inner:
        type: multisig
        owner: <signer>
        validators:
          44787:
            addrs: ['<signer>']
            threshold: 1
    ```

### Mock ISM (`mock`)

*   **Description:** A mock ISM that accepts all messages without any verification.
*   **When to use:** For testing and development purposes only. **DO NOT USE IN PRODUCTION.**
*   **Example:**
    ```yaml
    ism:
      type: mock
    ```

---

## Hooks

Hooks are contracts that are executed when a message is dispatched or processed. They are configured under the `hooks` key in the `deploy` section, typically within `default` and `required` aggregate hooks.

### Merkle Tree Hook (`merkle`)

*   **Description:** Adds every dispatched message to a Merkle tree. This is a core component of Hyperlane's security model.
*   **When to use:** This should be included in your `default` hook configuration for all standard deployments.
*   **Example:**
    ```yaml
    - type: merkle
    ```

### Interchain Gas Paymaster (IGP) Hook (`igp`)

*   **Description:** Facilitates paying for gas on the destination chain using tokens from the source chain.
*   **When to use:** Essential for most use cases to provide a smooth user experience, as it allows users to send messages without needing to hold gas tokens on the destination chain.
*   **Example:**
    ```yaml
    - type: igp
      owner: <signer>
      # The token to be used for gas payments. Defaults to the network's gas denom.
      token: uosmo
      configs:
        # Destination domain ID
        44787:
          exchange_rate: 1000000 # Token exchange rate
          gas_price: 10000000000 # Gas price on the destination chain in its smallest unit (e.g., wei)
      default_gas_usage: 30000
    ```

### Fee Hook (`fee`)

*   **Description:** Charges a specified fee to the sender of the message.
*   **When to use:** To monetize your relayer services or to cover operational costs.
*   **Example:**
    ```yaml
    - type: fee
      owner: <signer>
      fee:
        denom: uosmo
        amount: 1
    ```

### Pausable Hook (`pausable`)

*   **Description:** Allows an owner to pause the dispatching of new messages.
*   **When to use:** As a safety mechanism to halt message dispatching during an incident.
*   **Example:**
    ```yaml
    - type: pausable
      owner: <signer>
      paused: false
    ```

### Aggregate Hook (`aggregate`)

*   **Description:** Combines multiple hooks into a single hook. They are executed in the order they are listed.
*   **When to use:** This is the standard way to apply multiple hooks. You will typically have a `default` aggregate hook (for things like `merkle` and `igp`) and a `required` aggregate hook (for things like `pausable` and `fee`).
*   **Example:**
    ```yaml
    hooks:
      default:
        type: aggregate
        owner: <signer>
        hooks:
          - type: merkle
          - type: igp
            # ... igp config
      required:
        type: aggregate
        owner: <signer>
        hooks:
          - type: pausable
            # ... pausable config
    ```

### Routing Hooks (`routing`, `routing-custom`, `routing-fallback`)

*   **Description:** These hooks allow you to apply different hooks based on the destination of the message. `routing` uses the destination domain, while `routing-custom` uses the recipient address.
*   **When to use:** When you need fine-grained control over which hooks are applied to which messages.
*   **Example (`routing`):**
    ```yaml
    - type: "routing"
      owner: <signer>
      hooks:
        # Destination domain ID
        44787:
          type: fee
          owner: <signer>
          fee:
            denom: uosmo
            amount: 1
        # Another destination domain ID
        11155111:
          type: "mock"
    ```

### Mock Hook (`mock`)

*   **Description:** A mock hook that does nothing.
*   **When to use:** For testing and development purposes only.
*   **Example:**
    ```yaml
    - type: mock
    ```
