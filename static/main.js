(function() {
  const propertyHref = `../properties/updateRate${document.location.search}`;
  const imageHref = `current.jpg${document.location.search}`;
  let current = 1;
  let rate = null;
  let timer = null;

  function reloadImage() {
    fetch(imageHref, {
      headers: {
        Accept: 'image/*',
      },
      cache: 'reload',
    }).then((res) => {
      return res.blob();
    }).then((data) => {
      const el = document.querySelector(`#image-${current}`);
      current = current === 1 ? 2 : 1;
      el.src = URL.createObjectURL(data);

      document.querySelector('#image-1').classList.toggle('transparent');
      document.querySelector('#image-2').classList.toggle('transparent');
    }).catch(console.error);

    fetch(propertyHref, {
      headers: {
        Accept: 'application/json',
      },
      cache: 'reload',
    }).then((res) => {
      return res.json();
    }).then((data) => {
      if (data.updateRate !== rate) {
        rate = data.updateRate;

        if (timer !== null) {
          clearInterval(timer);
        }

        timer = setInterval(reloadImage, rate * 1000);
      }
    }).catch(console.error);
  }

  reloadImage();
})();
