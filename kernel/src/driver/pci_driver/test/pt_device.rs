use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
};

use crate::{
    driver::{
        base::{
            class::Class,
            device::{
                bus::Bus, driver::Driver, Device, DeviceKObjType, DeviceState, DeviceType, IdTable,
            },
            kobject::{KObjType, KObject, KObjectState, LockedKObjectState},
            kset::KSet,
        },
        pci_driver::{dev_id::PciDeviceID, pci_device::PciDevice, subsys::PciBus},
    },
    filesystem::{
        kernfs::KernFSInode,
        sysfs::{
            file::sysfs_emit_str, Attribute, AttributeGroup, SysFSOpsSupport, SYSFS_ATTR_MODE_RO,
        },
    },
    libs::{
        rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard},
        spinlock::SpinLock,
    },
};
#[derive(Debug)]
#[cast_to([sync] Device)]
#[cast_to([sync] PciDevice)]
/// 这是一个测试用的PciDevice，也可以作为新PciDevice的参考
/// 它需要实现KObject PciDevice Device这些接口
/// 并通过函数pci_device_manager().device_add（）来将设备进行接入
///
pub struct TestDevice {
    inner: RwLock<InnerPciDevice>,
    kobj_state: LockedKObjectState,
}

impl TestDevice {
    pub fn new() -> Self {
        let inner = RwLock::new(InnerPciDevice::default());

        Self {
            inner,
            kobj_state: LockedKObjectState::new(None),
        }
    }
}

impl PciDevice for TestDevice {
    fn dynid(&self) -> crate::driver::pci_driver::dev_id::PciDeviceID {
        PciDeviceID::dummpy()
    }
}

impl Device for TestDevice {
    fn attribute_groups(
        &self,
    ) -> Option<&'static [&'static dyn crate::filesystem::sysfs::AttributeGroup]> {
        Some(&[&HelloAttr])
    }

    fn bus(&self) -> Option<alloc::sync::Weak<dyn crate::driver::base::device::bus::Bus>> {
        self.inner.read().bus()
    }

    fn class(&self) -> Option<Arc<dyn Class>> {
        let mut guard = self.inner.write();
        let r = guard.class.clone()?.upgrade();
        if r.is_none() {
            guard.class = None;
        }

        return r;
    }

    fn driver(&self) -> Option<Arc<dyn Driver>> {
        self.inner.read().driver.clone()?.upgrade()
    }

    fn dev_type(&self) -> crate::driver::base::device::DeviceType {
        DeviceType::Pci
    }

    fn id_table(&self) -> crate::driver::base::device::IdTable {
        IdTable::new("testPci".to_string(), None)
    }

    fn can_match(&self) -> bool {
        true
    }

    fn is_dead(&self) -> bool {
        false
    }

    fn set_bus(&self, bus: Option<alloc::sync::Weak<dyn Bus>>) {
        self.inner.write().set_bus(bus);
    }

    fn set_can_match(&self, can_match: bool) {}

    fn set_class(&self, class: Option<alloc::sync::Weak<dyn Class>>) {
        self.inner.write().set_class(class)
    }

    fn set_driver(&self, driver: Option<alloc::sync::Weak<dyn Driver>>) {
        self.inner.write().set_driver(driver)
    }

    fn state_synced(&self) -> bool {
        true
    }
}

impl KObject for TestDevice {
    fn as_any_ref(&self) -> &dyn core::any::Any {
        self
    }

    fn set_inode(&self, inode: Option<Arc<KernFSInode>>) {
        self.inner.write().kern_inode = inode;
    }

    fn inode(&self) -> Option<Arc<KernFSInode>> {
        self.inner.read().kern_inode.clone()
    }

    fn parent(&self) -> Option<Weak<dyn KObject>> {
        self.inner.read().parent.clone()
    }

    fn set_parent(&self, parent: Option<Weak<dyn KObject>>) {
        self.inner.write().parent = parent;
    }

    fn kset(&self) -> Option<Arc<KSet>> {
        self.inner.read().kset.clone()
    }

    fn set_kset(&self, kset: Option<Arc<KSet>>) {
        self.inner.write().kset = kset;
    }

    fn kobj_type(&self) -> Option<&'static dyn KObjType> {
        self.inner.read().kobj_type
    }

    fn set_kobj_type(&self, ktype: Option<&'static dyn KObjType>) {
        self.inner.write().kobj_type = ktype;
    }

    fn name(&self) -> String {
        "PciTest".to_string()
    }

    fn set_name(&self, _name: String) {
        // do nothing
    }

    fn kobj_state(&self) -> RwLockReadGuard<KObjectState> {
        self.kobj_state.read()
    }

    fn kobj_state_mut(&self) -> RwLockWriteGuard<KObjectState> {
        self.kobj_state.write()
    }

    fn set_kobj_state(&self, state: KObjectState) {
        *self.kobj_state.write() = state;
    }
}
#[derive(Debug)]
pub struct InnerPciDevice {
    bus: Option<Weak<dyn Bus>>,
    pub name:Option<String>,
    pub class: Option<Weak<dyn Class>>,
    pub driver: Option<Weak<dyn Driver>>,
    pub kern_inode: Option<Arc<KernFSInode>>,
    pub parent: Option<Weak<dyn KObject>>,
    pub kset: Option<Arc<KSet>>,
    pub kobj_type: Option<&'static dyn KObjType>,
    device_state: DeviceState,
    pdev_id: i32,
    pdev_id_auto: bool,
}

impl InnerPciDevice {
    pub fn default() -> Self {
        Self {
            bus: None,
            class: None,
            name:None,
            driver: None,
            kern_inode: None,
            parent: None,
            kset: None,
            kobj_type: None,
            device_state: DeviceState::UnDefined,
            pdev_id: 0,
            pdev_id_auto: true,
        }
    }

    pub fn bus(&self) -> Option<Weak<dyn Bus>> {
        self.bus.clone()
    }

    pub fn class(&self) -> Option<Weak<dyn Class>> {
        self.class.clone()
    }

    pub fn driver(&self) -> Option<Weak<dyn Driver>> {
        self.driver.clone()
    }

    pub fn set_bus(&mut self, bus: Option<Weak<dyn Bus>>) {
        self.bus = bus
    }

    pub fn set_class(&mut self, class: Option<Weak<dyn Class>>) {
        self.class = class
    }

    pub fn set_driver(&mut self, driver: Option<Weak<dyn Driver>>) {
        self.driver = driver
    }

    
}

#[derive(Debug)]
pub struct HelloAttr;

impl AttributeGroup for HelloAttr {
    fn name(&self) -> Option<&str> {
        return Some("TestAttr");
    }

    fn attrs(&self) -> &[&'static dyn crate::filesystem::sysfs::Attribute] {
        &[&Hello]
    }

    fn is_visible(
        &self,
        kobj: Arc<dyn KObject>,
        attr: &'static dyn crate::filesystem::sysfs::Attribute,
    ) -> Option<crate::filesystem::vfs::syscall::ModeType> {
        return Some(attr.mode());
    }
}
#[derive(Debug)]
pub struct Hello;

impl Attribute for Hello {
    fn mode(&self) -> crate::filesystem::vfs::syscall::ModeType {
        SYSFS_ATTR_MODE_RO
    }

    fn name(&self) -> &str {
        "Hello"
    }

    fn show(
        &self,
        _kobj: Arc<dyn KObject>,
        _buf: &mut [u8],
    ) -> Result<usize, system_error::SystemError> {
        return sysfs_emit_str(_buf, &format!("Hello Pci"));
    }

    fn store(
        &self,
        _kobj: Arc<dyn KObject>,
        _buf: &[u8],
    ) -> Result<usize, system_error::SystemError> {
        todo!()
    }

    fn support(&self) -> crate::filesystem::sysfs::SysFSOpsSupport {
        SysFSOpsSupport::ATTR_SHOW
    }
}
