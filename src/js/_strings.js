/**
 * Turkish and English for the shell's own pages.
 *
 * The language is the machine's, chosen once in Rust: the application inside the
 * window speaks whatever the account is set to, but this page is drawn before
 * any account exists.
 */

const errors = {
  tr: {
    empty: "Bir adres yazın.",
    malformed: "Bu bir adres gibi görünmüyor.",
    scheme: "Yalnız http ve https adresleri açılabilir.",
    insecure: "Genel bir adrese http ile bağlanılamaz, çünkü oturum bilgisi açıkta gider. https yazın.",
    unreachable: "Sunucuya ulaşılamadı. Adresi ve ağ bağlantınızı denetleyin.",
    certificate: "Sunucunun güvenlik sertifikası doğrulanamadı. Sunucuyu kuran kişiye söyleyin.",
    "not-waterform": "Bu adreste bir WaterForm sunucusu yok.",
    "not-ready": "Sunucu şu an hizmet veremiyor. Birazdan yeniden deneyin.",
    unknown: "Bağlanılamadı.",
  },
  en: {
    empty: "Type an address.",
    malformed: "That does not look like an address.",
    scheme: "Only http and https addresses can be opened.",
    insecure: "Plain http is only for an address inside your own network, because the session would travel in the clear. Use https.",
    unreachable: "The server did not answer. Check the address and your network.",
    certificate: "The server's security certificate could not be verified. Tell whoever set the server up.",
    "not-waterform": "There is no WaterForm server at this address.",
    "not-ready": "The server cannot serve right now. Try again shortly.",
    unknown: "Could not connect.",
  },
};

export const dict = {
  tr: {
    connecting: "Bağlanılıyor",
    connectingButton: "Bağlanılıyor…",
    formTitle: "Sunucu adresi",
    formBody: "WaterForm bu adresteki sunucuda çalışır. Şirketinizin kendi sunucusu varsa adresini yazın.",
    fieldLabel: "Adres",
    connect: "Bağlan",
    cancel: "Vazgeç",
    formNote: "Bu ayarı sonra da değiştirebilirsiniz.",
    downTitle: "Sunucuya ulaşılamadı",
    retry: "Yeniden dene",
    change: "Adresi değiştir",
    errors: errors.tr,

    checking: "Güncellemeler denetleniyor",
    upToDateTitle: "Güncel",
    upToDateBody: "En son sürümü kullanıyorsunuz.",
    availableTitle: (v) => `WaterForm ${v} hazır`,
    availableBody: "Kurulum bir dakika sürer ve uygulama yeniden açılır. Sunucudaki işleriniz etkilenmez.",
    update: "Güncelle",
    later: "Sonra",
    downloading: "İndiriliyor",
    readyTitle: "Kuruldu",
    readyBody: "Yeni sürüm için uygulamayı yeniden başlatın.",
    restart: "Yeniden başlat",
    failedTitle: "Güncellenemedi",
    close: "Kapat",
  },
  en: {
    connecting: "Connecting",
    connectingButton: "Connecting…",
    formTitle: "Server address",
    formBody: "WaterForm runs on the server at this address. If your company has its own server, type its address.",
    fieldLabel: "Address",
    connect: "Connect",
    cancel: "Cancel",
    formNote: "You can change this later.",
    downTitle: "The server did not answer",
    retry: "Try again",
    change: "Change the address",
    errors: errors.en,

    checking: "Checking for updates",
    upToDateTitle: "Up to date",
    upToDateBody: "You are on the latest version.",
    availableTitle: (v) => `WaterForm ${v} is ready`,
    availableBody: "Installing takes about a minute and the application reopens. Your work on the server is untouched.",
    update: "Update",
    later: "Later",
    downloading: "Downloading",
    readyTitle: "Installed",
    readyBody: "Restart the application to use the new version.",
    restart: "Restart",
    failedTitle: "Could not update",
    close: "Close",
  },
};
