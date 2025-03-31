<div align="center">

  # 🍉 Komorice 🍚

  A komorebi GUI ricing configurator!

  [![Made with iced](https://iced.rs/badge.svg)](https://github.com/iced-rs/iced)

  ![image](assets/komorice.png)

  <image src="https://github.com/user-attachments/assets/78f1e400-6578-499b-8979-8438e6ed23c0" width="600"/>

</div>

## Overview

**Komorice** is a user-friendly graphical interface designed to simplify the configuration of the Windows Tiling Window
Manager, [**Komorebi**](https://github.com/LGUG2Z/komorebi). With Komorice, users can easily customize their tiling window
management experience, making it simple to create a productive workspace that fits their personal preferences.

## Usage

> [!Important]
> Before using komorice, make sure to backup your `komorebi.json` file!

Once you have Komorice up and running, it will try to automatically load your existing `komorebi.json` file.
If it says that it loaded your configuration file successfully then you can start modifying your Komorebi settings immediately.
Use the sidebar to navigate between different configuration categories and adjust the settings to your liking.
Be sure to save changes to see them reflected in your Komorebi setup.

### Pages

- #### **Home**:
  Is the starting screen. Here you can know if the config was properly loaded or not.
  
  If you which to start a config from scratch, you can remove your existing `komorebi.json` file (DON'T FORGET TO BACKUP BEFORE!).
  When komorice starts and can't find an existing file it will let you edit the config from scratch with all the defaults.
  When you save it, it will create the file for you.

> [!Note]
> In the future it is intended to let you load a specific file to edit
 
- #### **General**:
  Includes all the general and global config options.
- #### **Monitors**: It shows a real-time preview of your connected monitors and you can click any monitor to edit it.
  You can also toggle off the `Show Monitors` checkbox to be able to configure or add other monitor configs whitout being tied to the currently loaded monitors.
    - When editing a `Monitor` you are able to configure some configs on a per-monitor basis, like the padding or the work area offset.
    - You are also able to edit the monitor's `Workspaces`. When you press that button you get access to a list of the current workspaces and you can move, add or delete them as you see fit.
    When clicking on a `Workspace` you will enter that workspace config page, where you can change all the options specific to that workspace. You can also enter the `Workspace Rules`
    or `Initial Workspace Rules` to edit the rules for that workspace.
    - You can use the titles on top to navigate back.

  <details>
  <summary>Screenshots:</summary>
  
  ![image](https://github.com/user-attachments/assets/a1e5d3f2-d881-4765-bebb-4611491860b3)

  ![image](https://github.com/user-attachments/assets/cf7f6660-078c-4eb8-a138-190fa479bc20)
  *These images show how you can have multiple monitor configs, but only use some for the currently loaded monitors. In this case there are 3 monitor configs,
  the main monitor uses config index 0, while the second monitor uses config index 2. You set this up with the `Display Index Preferences`*
    
  **Monitor Screen:**
  ![image](https://github.com/user-attachments/assets/3ad732ee-989e-4963-ae63-4465e7ea2a66)

  **Workspaces Screen:**
  ![image](https://github.com/user-attachments/assets/5db58038-f237-4259-8585-4f45a021ae7e)

  **Workspace [0] - "I" Screen:**
  ![image](https://github.com/user-attachments/assets/a780157b-c244-45f9-ad64-26292fb7e3ec)

  **Workspace [0] - "I" Screen:** (Example of setting up `Window Container Behaviour Rules`)
  ![image](https://github.com/user-attachments/assets/881cf29e-a2ac-4942-866f-93d228a159a2)

  **Workspace [1] - "II" - Workspace Rules Screen:**
  ![image](https://github.com/user-attachments/assets/9725dcab-f034-4b0c-9c6e-2fa567ab3986)

  </details>
  
- #### **Border**:
  Edit all configs related to the borders.
  <details>
    <summary>Screenshots:</summary>

    **Border:** (Edit the border colours with a color picker)
    ![image](https://github.com/user-attachments/assets/11f4c208-8ebc-4243-b6a2-4c0167850778)

  </details>
- #### **Stackbar**:
  Edit all configs related to the stackbars. It shows a preview of the resulting stackbars on the bottom of this page.

  <details>
    <summary>Screenshots:</summary>
    
    **Stackbar:** (See a demo of how the stackbar will look like)
    ![image](https://github.com/user-attachments/assets/c3b27dd2-3c40-4e0c-b6ea-6a248c99ed0a)

  </details>
- #### **Transparency**:
  Edit all configs related to the transparency, including the `Transparency Ignore Rules`.
- #### **Animations**:
  Edit all configs related to animations.
  <details>
    <summary>Screenshots:</summary>

    **Animations:** (Edit per type)
    ![image](https://github.com/user-attachments/assets/dda59fe8-a31c-4f2e-9346-4cba21bb5387)

    **Animations:** (Or glabally)
    ![image](https://github.com/user-attachments/assets/315ccf4b-a4d6-4e5f-8844-366069c29263)

  </details>
- #### **Theme**:
  Set or edit the komorebi global theme. Be aware that using this will override your border colors from **Border**.
- #### **Rules**:
  Set or edit all your komorebi application rules.

  You can easily create, edit, copy, paste or remove your rules. When you copy a rule it copies it with the correct JSON
  format to be able to be pasted on a `komorebi.json` file directly. So you can eaasily copy some rule and share it with
  others on Discord. The same way, if you copy a rule from someone else on Discord or on their `komorebi.json` file, komorice
  will recognize that the contents of your clipboard can be serialized to a rule and will show the paste button when you are
  creating a new rule.
  
  <details>
    <summary>Screenshots:</summary>

    **Rules:**
    ![image](https://github.com/user-attachments/assets/cc6e1e38-3fc2-459a-b024-8c0c75b8a1bc)

    **Rules:** (You can edit, copy or remove existing rules)
    ![image](https://github.com/user-attachments/assets/314c493a-c85d-44f4-9488-fd14268eec55)

    **Rules:** (You can easily create new rules)
    ![image](https://github.com/user-attachments/assets/64b3e14b-a6d8-4836-8e2d-1b051e9a70b8)

    **Rules:** (Paste button active so that you can paste a rule from your clipboard)
    ![image](https://github.com/user-attachments/assets/6524f1a4-2d73-410a-bcee-5fde769a4603)

  </details>
  
- #### **Live Debug**:
  WIP (It will let you act on your komorebi immediately without changing your config, somewhat like the `komorebi-gui` lets you do now...)
- #### **Settings**:
  Change the komorice app settings.

## Installation

To get started with Komorice, follow these steps:

1. Install from cargo:
   ```
   cargo install --git https://github.com/alex-ds13/komorice.git
   ```
2. Run the application:
   ```
   komorice.exe
   ```

## License

`komorice` is licensed under the [Komorebi 2.0.0 license](https://github.com/LGUG2Z/komorebi-license), which is a fork of the
[PolyForm Strict 1.0.0 license](https://polyformproject.org/licenses/strict/1.0.0). On a high level this means that you are free
to do whatever you want with `komorice` for personal use other than redistribution, or distribution of new works (i.e. hard-forks)
based on the software.

Anyone is free to make their own fork of `komorice` with changes intended either for personal use or for integration back upstream
via pull requests.

*The [Komorebi 2.0.0 License](./LICENSE.md) does not permit any kind of commercial use.*

### Contribution licensing

Contributions are accepted with the following understanding:

- Contributed content is licensed under the terms of the 0-BSD license
- Contributors accept the terms of the project license at the time of contribution

By making a contribution, you accept both the current project license terms, and that all contributions that you have
made are provided under the terms of the 0-BSD license.

<details>
<summary><h4>0-BSD license</h4></summary>  

  #### Zero-Clause BSD

```
Permission to use, copy, modify, and/or distribute this software for
any purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED “AS IS” AND THE AUTHOR DISCLAIMS ALL
WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES
OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE
FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY
DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```
</details>

## Support

If you have any issues, questions, or feedback regarding Komorice, feel free to open an issue in the GitHub repository.

## Acknowledgements

Thanks to the Komorebi community and specially to [@LGUG2Z](https://github.com/LGUG2Z) who have made this project possible!
