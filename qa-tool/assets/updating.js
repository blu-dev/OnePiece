var currentCardId = "";
function processCardUpdate(json) {
  currentCardId = json.id;
  document.getElementById("cardPreview").src = `images/${json.id}`;
  document.getElementById("cardname").value = json.name;
  document.getElementById("cost_life").value = json.cost_life;

  var has_power = document.getElementById("haspower");
  var power = document.getElementById("power");
  has_power.checked = json.power !== null;
  if (has_power.checked) {
    power.disabled = false;
    power.value = json.power;
  } else {
    power.disabled = true;
    power.value = 0;
  }

  var has_counter = document.getElementById("hascounter");
  var counter = document.getElementById("counter");
  has_counter.checked = json.counter !== null;
  if (has_counter.checked) {
    counter.disabled = false;
    counter.value = json.counter;
  } else {
    counter.disabled = true;
    counter.value = 0;
  }

  var has_effect = document.getElementById("haseffect");
  var effect = document.getElementById("effect");
  has_effect.checked = json.effect !== null;
  if (has_effect.checked) {
    effect.disabled = false;
    effect.value = json.effect;
  } else {
    effect.disabled = true;
    effect.value = "";
  }

  var has_trigger = document.getElementById("hastrigger");
  var trigger = document.getElementById("trigger");
  has_trigger.checked = json.trigger !== null;
  if (has_trigger.checked) {
    trigger.disabled = false;
    trigger.value = json.trigger;
  } else {
    trigger.disabled = true;
    trigger.value = "";
  }

  var attributes = "";
  for (const attribute of json.attributes) {
    if (attributes === "") {
      attributes = attribute;
    } else {
      attributes = attributes + "/" + attribute;
    }
  }

  document.getElementById("attributes").value = attributes;

  var colors = "";
  for (const color of json.colors) {
    if (colors === "") {
      colors = color;
    } else {
      colors = colors + "/" + color;
    }
  }

  document.getElementById("colors").value = colors;

  var subtypes = "";
  for (const subtype of json.subtypes) {
    if (subtypes === "") {
      subtypes = subtype;
    } else {
      subtypes = subtypes + "/" + subtype;
    }
  }

  document.getElementById("subtypes").value = subtypes;
}

document.addEventListener("DOMContentLoaded", async function (_) {
  fetch("http://localhost:8080/start").then((response) =>
    response.json().then(processCardUpdate),
  );

  document.getElementById("prevCard").onclick = function () {
    fetch(`http://localhost:8080/prev/${currentCardId}`).then((response) =>
      response.json().then(processCardUpdate),
    );
  };

  document.getElementById("submit").onclick = function () {
    const subtypes = document.getElementById("subtypes").value.split("/");
    const colors = document.getElementById("colors").value.split("/");
    const attributes = document.getElementById("attributes").value.split("/");
    const body = JSON.stringify({
      id: currentCardId,
      name: document.getElementById("cardname").value,
      ty: document.getElementById("cardkind").value,
      subtypes: subtypes,
      colors: colors,
      attributes: attributes,
      cost_life: document.getElementById("cost_life").value,
      power: document.getElementById("haspower").checked
        ? document.getElementById("power")
        : null,
      counter: document.getElementById("hascounter").checked
        ? document.getElementById("counter")
        : null,
      effect: document.getElementById("haseffect").checked
        ? document.getElementById("effect").value
        : null,
      trigger: document.getElementById("hastrigger").checked
        ? document.getElementById("trigger").value
        : null,
    });

    fetch(`http://localhost:8080/submit`, {
      method: "POST",
      body: body,
    });
  };

  document.getElementById("nextCard").onclick = function () {
    fetch(`http://localhost:8080/next/${currentCardId}`).then((response) =>
      response.json().then(processCardUpdate),
    );
  };

  document.getElementById("haspower").onchange = function () {
    var has_power = document.getElementById("haspower").checked;
    var power = document.getElementById("power");
    if (has_power) {
      power.disabled = false;
    } else {
      power.disabled = true;
    }
  };

  document.getElementById("hascounter").onchange = function () {
    var has_counter = document.getElementById("hascounter").checked;
    var counter = document.getElementById("counter");
    if (has_counter) {
      counter.disabled = false;
    } else {
      counter.disabled = true;
    }
  };

  document.getElementById("haseffect").onchange = function () {
    var has_effect = document.getElementById("haseffect").checked;
    var effect = document.getElementById("effect");
    if (has_effect) {
      effect.disabled = false;
    } else {
      effect.disabled = true;
    }
  };

  document.getElementById("hastrigger").onchange = function () {
    var has_trigger = document.getElementById("hastrigger").checked;
    var trigger = document.getElementById("trigger");
    if (has_trigger) {
      trigger.disabled = false;
    } else {
      trigger.disabled = true;
    }
  };
});
