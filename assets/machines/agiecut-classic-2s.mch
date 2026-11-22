{
  "general": {
    "model": "AGIECUT CLASSIC 2S",
    "vendor": "Camcraft",
    "kind": "wire_edm",
    "description": "Machine setup template from AGIE technical data C2.1, June 2002, sections 1.1 and 5.2. X/Y/Z stroke lengths are 350/250/256 mm. Set the travel endpoints and guide heights for the installed machine. Add the cutting conditions from its control and select a compatible post before use."
  },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "x", "name": "X", "type": "linear", "parts": [
        { "id": "y", "name": "Y", "type": "linear", "parts": [
          { "id": "table", "name": "table", "type": "table" }
        ] }
      ] },
      { "id": "z", "name": "Z", "type": "linear", "parts": [
        { "id": "u", "name": "U", "type": "linear", "travel": { "min": -70.0, "max": 70.0 }, "parts": [
          { "id": "v", "name": "V", "type": "linear", "travel": { "min": -70.0, "max": 70.0 }, "parts": [
            { "id": "head", "name": "upper guide", "type": "head" }
          ] }
        ] }
      ] }
    ]
  } },
  "controller": { "synced_configuration": { "parts": {
    "X": { "coordinate": 0, "max_rapid_speed": 900.0 },
    "Y": { "coordinate": 1, "max_rapid_speed": 900.0 },
    "Z": { "coordinate": 2, "max_rapid_speed": 600.0 },
    "U": { "coordinate": 3, "max_rapid_speed": 600.0 },
    "V": { "coordinate": 4, "max_rapid_speed": 600.0 }
  } } },
  "edm": { "wire_diameters": [0.20, 0.25, 0.30, 0.33] }
}
